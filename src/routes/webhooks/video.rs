use rocket::http::Status;
use rocket_contrib::json::Json;

use log::warn;

use crate::database::DatabaseConnection;
use crate::models::user::get_user_by_id;
use crate::services::encoder_service::{self, Job};
use crate::services::notification_service;

#[rocket::post("/webhooks/video?<key>", format = "json", data = "<request>")]
pub(crate) fn webhook(conn: DatabaseConnection, request: Json<Job>, key: Option<String>) -> Status {
    if key.is_none() {
        return Status::BadRequest;
    }

    let video_encoding_key = key.unwrap();

    match encoder_service::accept_webhook(&conn, &video_encoding_key, &request.into_inner()) {
        Ok(upload) => {
            upload
                .uploader_user_id
                .and_then(|uploader_user_id| get_user_by_id(&conn, uploader_user_id))
                .and_then(|user| {
                    notification_service::notify_new_upload(&upload, &user);
                    Some(())
                });

            Status::Ok
        }
        Err(err) => {
            warn!("/webhooks/video: {:?}", err);
            Status::BadRequest
        }
    }
}
