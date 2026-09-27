// This module handles integration with our 3rd-party video encoding service (Coconut API v2).
// https://docs.coconut.co/jobs/api

use std::env;

use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::database::DatabaseConnection;
use crate::models::upload::{self, FinishedEncodingUpload, Upload, UploadStatus};
use crate::models::user::get_user_by_id;

const API_URL: &'static str = "https://api.coconut.co/v2/jobs";
const BUCKET: &'static str = "bits.spin-archive.org";
const REGION: &'static str = "us-west-1";
const WEBHOOK_URL: &'static str = "https://spin-archive.org/webhooks/video";

/// A job, as returned when it is created.
#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub status: Option<String>,
}

/// The webhook payload Coconut posts when a job finishes.
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

/// Enqueues an upload to be transcoded.
pub fn enqueue_upload(upload: &Upload) -> Result<Job, EncoderError> {
    let api_key = env::var("COCONUT_API_KEY").unwrap_or_default();

    let video_path = format!("/e/{}.mp4", upload.file_id);
    let thumbnail_path = format!("/t/{}.jpg", upload.file_id);

    // Encode a single MP4, picking the resolution from the source width.
    let config = json!({
        "input": {
            "url": upload.get_file_url(),
        },
        "storage": {
            "service": "wasabi",
            "bucket": BUCKET,
            "region": REGION,
            "credentials": {
                "access_key_id": env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
                "secret_access_key": env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            },
        },
        "notification": {
            "type": "http",
            "url": WEBHOOK_URL,
            "params": {
                "key": upload.video_encoding_key,
            },
        },
        "outputs": {
            "mp4:480p": {
                "key": "mp4:480p",
                "path": video_path,
                "if": "{{ input.width }} < 1280",
            },
            "mp4:720p": {
                "key": "mp4:720p",
                "path": video_path,
                "if": "{{ input.width }} >= 1280 AND {{ input.width }} < 1980",
            },
            "mp4:1080p": {
                "key": "mp4:1080p",
                "path": video_path,
                "if": "{{ input.width }} >= 1980",
            },
            "jpg:300x": {
                "key": "jpg:thumbnail",
                "path": thumbnail_path,
                "number": 1,
            },
        },
    });

    let client = reqwest::blocking::Client::new();

    let response = client
        .post(API_URL)
        .basic_auth(api_key, Some(""))
        .json(&config)
        .send()
        .map_err(|e| {
            warn!("[encoding] request failed: {:?}", e);
            EncoderError::ApiFailure
        })?;

    if !response.status().is_success() {
        warn!(
            "[encoding] job rejected ({}): {}",
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
