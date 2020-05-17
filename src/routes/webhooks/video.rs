use rocket::http::Status;
use rocket_contrib::json::Json;

use log::warn;

use crate::database::DatabaseConnection;
use crate::services::encoder_service::{self, Job};

#[rocket::post("/webhooks/video?<key>", format = "json", data = "<request>")]
pub(crate) fn webhook(conn: DatabaseConnection, request: Json<Job>, key: Option<String>) -> Status {
  if key.is_none() {
    return Status::BadRequest;
  }

  let video_encoding_key = key.unwrap();

  match encoder_service::accept_webhook(&conn, &video_encoding_key, &request.into_inner()) {
    Ok(_) => Status::Ok,
    Err(err) => {
      warn!("/webhooks/video: {:?}", err);
      Status::BadRequest
    }
  }
}
