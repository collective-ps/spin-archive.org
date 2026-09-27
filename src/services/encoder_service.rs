// This module handles video encoding. Each upload is encoded by a single-use Fly Machine
// running the image in `encoder/`, started through the Machines API. The Machine writes its
// outputs to presigned Wasabi URLs and reports back to `/webhooks/video`.
// https://docs.fly.io/machines/api/
//
// Finalized uploads are queued (Processing, no `encoder_machine_id`). The dispatcher starts
// Machines for them while fewer than `max_concurrent_encodes()` are running, spacing out
// requests to stay under the Machines API rate limit (1 create/s per app). It runs after
// each finalize and webhook, and periodically in the background.

use std::env;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use lazy_static::lazy_static;

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::database::{self, DatabaseConnection};
use crate::models::upload::{self, FinishedEncodingUpload, Upload, UploadStatus};
use crate::models::user::get_user_by_id;
use crate::s3_client;

const WEBHOOK_URL: &'static str = "https://spin-archive.org/webhooks/video";

/// How long the encoder has to upload its outputs.
const UPLOAD_URL_TTL: Duration = Duration::from_secs(60 * 60 * 12);

/// Encodes that haven't reported back after this long are marked failed. The encoder
/// itself gives up after 3 hours.
const STALE_ENCODE_AFTER: Duration = Duration::from_secs(60 * 60 * 3 + 60 * 15);

/// Gap between Machine creations, to stay under the Machines API rate limit.
const CREATE_INTERVAL: Duration = Duration::from_millis(1100);

/// How often the background dispatcher checks the queue.
const DISPATCH_INTERVAL: Duration = Duration::from_secs(60);

lazy_static! {
    /// Only one dispatcher runs at a time, so concurrent triggers can't overshoot the limit.
    static ref DISPATCH_LOCK: Mutex<()> = Mutex::new(());
}

/// The encoder Machine, as returned when it is created.
#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub state: Option<String>,
}

/// The webhook payload the encoder posts when it finishes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub job_id: String,
    pub event: String,
    pub data: Value,
}

#[derive(Debug)]
pub enum EncoderError {
    ApiFailure,
    JsonError,
    UploadNotFound,
}

fn machines_api() -> String {
    env::var("ENCODER_MACHINES_API").unwrap_or("https://api.machines.dev/v1".to_owned())
}

fn max_concurrent_encodes() -> i64 {
    env::var("ENCODER_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3)
}

fn encoder_app() -> String {
    env::var("ENCODER_APP").unwrap_or("spin-archive-encoder".to_owned())
}

/// Authorization header for the Machines API. `fly tokens create` output starts with
/// "FlyV1 " and is sent as-is; anything else is treated as a bearer token.
fn authorization() -> String {
    let token = env::var("FLY_ENCODER_TOKEN").unwrap_or_default();

    if token.starts_with("FlyV1 ") {
        token
    } else {
        format!("Bearer {}", token)
    }
}

/// Starts an encoder Machine for an upload, bypassing the queue.
pub fn start_encoder(upload: &Upload) -> Result<Job, EncoderError> {
    let app = encoder_app();
    let image =
        env::var("ENCODER_IMAGE").unwrap_or(format!("registry.fly.io/{}:latest", app));

    let video_put_url = s3_client::generate_signed_put_url(
        &format!("e/{}.mp4", upload.file_id),
        "video/mp4",
        UPLOAD_URL_TTL,
    );
    let thumbnail_put_url = s3_client::generate_signed_put_url(
        &format!("t/{}.jpg", upload.file_id),
        "image/jpeg",
        UPLOAD_URL_TTL,
    );

    let machine = json!({
        "region": "sjc",
        "config": {
            "image": image,
            "env": {
                "INPUT_URL": upload.get_file_url(),
                "VIDEO_PUT_URL": video_put_url,
                "THUMBNAIL_PUT_URL": thumbnail_put_url,
                "WEBHOOK_URL": format!("{}?key={}", WEBHOOK_URL, upload.video_encoding_key),
            },
            "guest": {
                "cpu_kind": "performance",
                "cpus": 1,
                "memory_mb": 2048,
            },
            "auto_destroy": true,
            "restart": { "policy": "no" },
            "metadata": { "upload": upload.file_id },
        },
    });

    let client = reqwest::blocking::Client::new();

    let response = client
        .post(&format!("{}/apps/{}/machines", machines_api(), app))
        .header("Authorization", authorization())
        .json(&machine)
        .send()
        .map_err(|e| {
            warn!("[encoding] request failed: {:?}", e);
            EncoderError::ApiFailure
        })?;

    if !response.status().is_success() {
        warn!(
            "[encoding] could not start encoder ({}): {}",
            response.status(),
            response.text().unwrap_or_default()
        );

        return Err(EncoderError::ApiFailure);
    }

    response.json::<Job>().map_err(|err| {
        warn!("[encoding] {:?}", err);
        EncoderError::JsonError
    })
}

/// An encoder Machine, as listed by the Machines API.
#[derive(Debug, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub state: String,
    pub region: String,
    pub created_at: String,
    #[serde(default)]
    pub config: Value,
}

impl Machine {
    /// The upload this Machine is encoding.
    pub fn upload_file_id(&self) -> Option<&str> {
        self.config["metadata"]["upload"].as_str()
    }
}

/// Lists the encoder app's Machines. Finished Machines destroy themselves, so these are
/// the ones still starting or running.
pub fn list_machines() -> Result<Vec<Machine>, EncoderError> {
    let response = reqwest::blocking::Client::new()
        .get(&format!("{}/apps/{}/machines", machines_api(), encoder_app()))
        .header("Authorization", authorization())
        .send()
        .map_err(|e| {
            warn!("[encoding] could not list machines: {:?}", e);
            EncoderError::ApiFailure
        })?;

    if !response.status().is_success() {
        warn!("[encoding] could not list machines ({})", response.status());
        return Err(EncoderError::ApiFailure);
    }

    response.json::<Vec<Machine>>().map_err(|e| {
        warn!("[encoding] {:?}", e);
        EncoderError::JsonError
    })
}

/// Starts encoders for queued uploads while there are free slots.
pub fn dispatch(conn: &DatabaseConnection) {
    let _lock = DISPATCH_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    let stale_cutoff = Utc::now().naive_utc()
        - chrono::Duration::from_std(STALE_ENCODE_AFTER).unwrap_or(chrono::Duration::hours(4));

    match upload::fail_stale_encodings(&conn, stale_cutoff) {
        Ok(0) => (),
        Ok(count) => warn!("[encoding] marked {} stale encodes as failed", count),
        Err(e) => warn!("[encoding] could not check for stale encodes: {:?}", e),
    }

    let free_slots = max_concurrent_encodes() - upload::count_encoding(&conn);

    if free_slots <= 0 {
        return;
    }

    for (i, queued) in upload::get_queued_for_encoding(&conn, free_slots)
        .iter()
        .enumerate()
    {
        if i > 0 {
            thread::sleep(CREATE_INTERVAL);
        }

        match start_encoder(queued) {
            Ok(job) => {
                debug!("[encoding] started {} for upload {}", job.id, queued.file_id);

                if let Err(e) = upload::set_encoder_machine(
                    &conn,
                    queued.id,
                    &job.id,
                    Utc::now().naive_utc(),
                ) {
                    warn!("[encoding] could not record machine {}: {:?}", job.id, e);
                }
            }
            // Leave it queued; the next dispatch retries.
            Err(_) => break,
        }
    }
}

/// Runs [`dispatch`] on a background thread.
pub fn dispatch_in_background() {
    thread::spawn(|| {
        if let Some(conn) = database::background_connection() {
            dispatch(&conn);
        }
    });
}

/// Dispatches now and then every `DISPATCH_INTERVAL`, so queued uploads are picked up
/// after restarts and after failed Machine creations.
pub fn spawn_dispatcher() {
    thread::spawn(|| loop {
        if let Some(conn) = database::background_connection() {
            dispatch(&conn);
        }

        thread::sleep(DISPATCH_INTERVAL);
    });
}

pub fn accept_webhook(
    conn: &DatabaseConnection,
    video_encoding_key: &str,
    notification: &Notification,
) -> Result<Upload, EncoderError> {
    let upload = match upload::get_by_video_encoding_key(&conn, video_encoding_key) {
        Some(upload) => upload,
        None => return Err(EncoderError::UploadNotFound),
    };

    match notification.event.as_str() {
        "job.completed" => {
            let video_url = format!("https://bits.spin-archive.org/e/{}.mp4", upload.file_id);
            let thumbnail_url = format!("https://bits.spin-archive.org/t/{}.jpg", upload.file_id);

            let uploader = get_user_by_id(&conn, upload.uploader_user_id.unwrap()).unwrap();

            let status = if uploader.is_contributor() {
                UploadStatus::Completed
            } else {
                UploadStatus::PendingApproval
            };

            let finished_encoding = FinishedEncodingUpload {
                status,
                thumbnail_url,
                video_url,
            };

            upload::update_encoding(&conn, upload.id, &finished_encoding)
                .map_err(|_| EncoderError::ApiFailure)
        }
        "job.failed" => {
            warn!(
                "[encoding] job {} failed for upload {}: {}",
                notification.job_id, upload.file_id, notification.data
            );

            upload::update_status(&conn, upload.id, UploadStatus::Failed)
                .map_err(|_| EncoderError::ApiFailure)
        }
        _ => Err(EncoderError::ApiFailure),
    }
}
