use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseConnection;
use crate::models::user::User;
use crate::s3_client::generate_signed_url;
use crate::services::upload_service;

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
  id: String,
  url: String,
}

#[rocket::post("/upload")]
pub(crate) fn upload(
  conn: DatabaseConnection,
  user: &User,
) -> Result<Json<UploadResponse>, BadRequest<()>> {
  if !user.can_upload() {
    return Err(BadRequest(None));
  }

  match upload_service::new_pending_upload(&conn) {
    Ok(upload) => Ok(Json(UploadResponse {
      id: upload.id.to_string().to_owned(),
      url: generate_signed_url(&upload),
    })),
    Err(_) => Err(BadRequest(None)),
  }
}
