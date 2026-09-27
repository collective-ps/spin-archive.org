// This module handles video encoding. Each upload is encoded by a single-use Fly Machine
// running the image in `encoder/`, started through the Machines API. The Machine writes its
// outputs to presigned Wasabi URLs and reports back to `/webhooks/video`.
// https://docs.fly.io/machines/api/

use std::env;
use std::time::Duration;

use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::database::DatabaseConnection;
use crate::models::upload::{self, FinishedEncodingUpload, Upload, UploadStatus};
use crate::models::user::get_user_by_id;
use crate::s3_client;

const MACHINES_API: &'static str = "https://api.machines.dev/v1";
const WEBHOOK_URL: &'static str = "https://spin-archive.org/webhooks/video";

/// How long the encoder has to upload its outputs.
const UPLOAD_URL_TTL: Duration = Duration::from_secs(60 * 60 * 12);

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

/// Enqueues an upload to be transcoded.
pub fn enqueue_upload(upload: &Upload) -> Result<Job, EncoderError> {
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
        .post(&format!("{}/apps/{}/machines", MACHINES_API, app))
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
