use serde_json::json;

use crate::config;
use crate::models::{upload::Upload, upload_comment::UploadComment, user::User};

/// Notify Contributor Discord that a new pending upload has been submitted for approval.
pub fn notify_pending_upload(upload: &Upload, user: &User) {
    let thumbnail_url = upload
        .thumbnail_url
        .clone()
        .unwrap_or("http://bits.spin-archive.org/placeholder.jpg".to_string());
    let url = format!("https://spin-archive.org/u/{}", upload.file_id);
    let uploader_name = &user.username;
    let tag_string = &upload.tag_string;

    let json = json!({
      "embeds": [
        {
          "title": "Uploaded a video for approval.",
          "description": format!("`{}`", upload.file_name.as_ref().unwrap_or(&"No original file name.".to_string())),
          "image": {
            "url": thumbnail_url,
          },
          "footer": {
            "text": tag_string,
          },
          "url": url,
          "color": 7506394,
          "author": {
            "name": uploader_name,
          }
        }
      ]
    });

    let client = reqwest::blocking::Client::new();
    let webhook_url = config::get_contributor_webhook_url();

    let _ = client.post(&webhook_url).json(&json).send();
}

/// Notify Discord that a new upload has been completed.
pub fn notify_new_upload(upload: &Upload, user: &User) {
    let thumbnail_url = upload
        .thumbnail_url
        .clone()
        .unwrap_or("http://bits.spin-archive.org/placeholder.jpg".to_string());
    let url = format!("https://spin-archive.org/u/{}", upload.file_id);
    let uploader_name = &user.username;
    let tag_string = &upload.tag_string;

    let json = json!({
      "embeds": [
        {
          "title": "Uploaded a new video.",
          "description": format!("`{}`", upload.file_name.as_ref().unwrap_or(&"No original file name.".to_string())),
          "image": {
            "url": thumbnail_url,
          },
          "footer": {
            "text": tag_string,
          },
          "url": url,
          "color": 7506394,
          "author": {
            "name": uploader_name,
          }
        }
      ]
    });

    let client = reqwest::blocking::Client::new();
    let webhook_url = config::get_webhook_url();

    let _ = client.post(&webhook_url).json(&json).send();
}

pub fn notify_new_comment(comment: &UploadComment, upload: &Upload, user: &User) {
    let thumbnail_url = upload
        .thumbnail_url
        .clone()
        .unwrap_or("http://bits.spin-archive.org/placeholder.jpg".to_string());
    let url = format!("https://spin-archive.org/u/{}", upload.file_id);
    let uploader_name = &user.username;
    let tag_string = &upload.tag_string;

    let json = json!({
      "embeds": [
        {
          "title": format!("Commented on {} - #{}.", upload.file_name.as_ref().unwrap_or(&"unknown".to_string()), upload.file_id),
          "description": comment.comment,
          "image": {
            "url": thumbnail_url,
          },
          "footer": {
            "text": tag_string,
          },
          "url": url,
          "color": 7506394,
          "author": {
            "name": uploader_name,
          }
        }
      ]
    });

    let client = reqwest::blocking::Client::new();
    let webhook_url = config::get_webhook_url();

    let _ = client.post(&webhook_url).json(&json).send();
}
