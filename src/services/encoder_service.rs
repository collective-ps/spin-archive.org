// This module handles integration with our 3rd-party video encoding service.

use std::env;

use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::DatabaseConnection;
use crate::models::upload::{self, FinishedEncodingUpload, Upload, UploadStatus};

const HOST: &'static str = "https://s3.us-west-1.wasabisys.com";
const BUCKET: &'static str = "bits.spin-archive.org";
const WEBHOOK_BASE: &'static str = "https://spin-archive.org/webhooks/video";

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
  id: i32,
  status: Option<String>,
  created_at: Option<String>,
  completed_at: Option<String>,
  progress: Option<String>,
  errors: Value,
  output_urls: Value,
  event: Option<String>,
}

#[derive(Debug)]
pub enum EncoderError {
  ApiFailure,
  JsonError,
  UploadNotFound,
}

/// Enqueues an upload to be transcoded.
pub fn enqueue_upload(upload: &Upload) -> Result<Job, EncoderError> {
  let api_key = env::var("COCONUT_API_KEY").unwrap();

  let source = upload.get_file_url();

  let output_filename = format!("{file_id}.mp4", file_id = upload.file_id,);

  let webhook_url = format!("{}?key={}", WEBHOOK_BASE, upload.video_encoding_key);

  let config = vec![
    format!("set source = {}", source),
    format!("set webhook = {}", webhook_url),
    format!(
      "-> mp4 = {}, keep=video_bitrate,audio_bitrate, if=$source_video_bitrate <= 8000",
      output_url("e", &output_filename)
    ),
    format!(
      "-> mp4::quality=4 = {}, keep=audio_bitrate, if=$source_video_bitrate > 8000",
      output_url("e", &output_filename)
    ),
    format!(
      "-> jpg:300x = {}",
      output_url("t", &format!("{}.jpg", upload.file_id))
    ),
  ]
  .join("\n");

  let client = reqwest::blocking::Client::new();

  match client
    .post("https://api.coconut.co/v1/job")
    .basic_auth(api_key, None::<String>)
    .body(config)
    .send()
  {
    Ok(response) => response.json::<Job>().map_err(|err| {
      warn!("{:?}", err);
      EncoderError::JsonError
    }),
    Err(e) => {
      warn!("{:?}", e);

      Err(EncoderError::ApiFailure)
    }
  }
}

pub fn accept_webhook(
  conn: &DatabaseConnection,
  video_encoding_key: &str,
  job: &Job,
) -> Result<Upload, EncoderError> {
  match upload::get_by_video_encoding_key(&conn, video_encoding_key) {
    Some(upload) => {
      if job.event == Some("job.completed".to_string()) {
        let output_filename = format!("{file_id}.mp4", file_id = upload.file_id,);
        let video_url = format!("https://bits.spin-archive.org/e/{}", output_filename);
        let thumbnail_url = format!("https://bits.spin-archive.org/t/{}.jpg", upload.file_id);

        let finished_encoding = FinishedEncodingUpload {
          status: UploadStatus::Completed,
          thumbnail_url: thumbnail_url,
          video_url: video_url,
        };

        match upload::update_encoding(&conn, upload.id, &finished_encoding) {
          Ok(upload) => Ok(upload),
          Err(_) => Err(EncoderError::ApiFailure),
        }
      } else {
        Err(EncoderError::ApiFailure)
      }
    }
    None => Err(EncoderError::UploadNotFound),
  }
}

fn output_url(prefix: &str, file_name: &str) -> String {
  let access_key = env::var("AWS_ACCESS_KEY_ID").unwrap();
  let secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap();

  format!(
    "s3://{access_key}:{secret_key}@{bucket}/{prefix}/{output}?host={host}",
    access_key = access_key,
    secret_key = secret_key,
    bucket = BUCKET,
    prefix = prefix,
    output = file_name,
    host = HOST
  )
}
