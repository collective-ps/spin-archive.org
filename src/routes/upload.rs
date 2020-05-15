use std::collections::HashMap;
use std::path::Path;

use rocket::request::FlashMessage;
use rocket::response::status::BadRequest;
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::User;
use crate::s3_client::generate_signed_url;
use crate::services::upload_service;

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
  id: String,
  url: String,
  thumbnail_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
  file_name: String,
  content_length: i32,
}

#[derive(Serialize, Deserialize)]
pub struct FinalizeUploadResponse {}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct FinalizeUploadRequest {
  tags: String,
  source: String,
}

#[rocket::get("/upload")]
pub(crate) fn index(flash: Option<FlashMessage>, user: &User) -> Result<Template, Redirect> {
  if user.can_upload() {
    let mut context: HashMap<String, String> = HashMap::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, Some(user));

    Ok(Template::render("upload", &context))
  } else {
    Err(Redirect::to("/"))
  }
}

#[rocket::get("/upload", rank = 2)]
pub(crate) fn index_not_logged_in() -> Redirect {
  Redirect::to("/")
}

#[rocket::post("/upload", format = "json", data = "<request>")]
pub(crate) fn upload(
  conn: DatabaseConnection,
  user: &User,
  request: Json<UploadRequest>,
) -> Result<Json<UploadResponse>, BadRequest<()>> {
  if !user.can_upload() {
    return Err(BadRequest(None));
  }

  let path = Path::new(&request.file_name);
  let name = path.file_name();
  let ext = path.extension();

  if name.is_none() || ext.is_none() {
    return Err(BadRequest(None));
  }

  match upload_service::new_pending_upload(
    &conn,
    &user,
    name.unwrap().to_str().unwrap(),
    ext.unwrap().to_str().unwrap(),
  ) {
    Ok(upload) => {
      let file_name = format!("{}.{}", &upload.file_id, &upload.file_ext);
      let thumbnail_name = format!("{}.jpg", &upload.file_id);

      Ok(Json(UploadResponse {
        id: upload.file_id.to_owned(),
        url: generate_signed_url(&file_name),
        thumbnail_url: generate_signed_url(&thumbnail_name),
      }))
    }
    Err(_) => Err(BadRequest(None)),
  }
}

#[rocket::post("/upload/<file_id>/finalize", format = "json", data = "<request>")]
pub(crate) fn finalize(
  conn: DatabaseConnection,
  user: &User,
  file_id: String,
  request: Json<FinalizeUploadRequest>,
) -> Result<Json<FinalizeUploadResponse>, BadRequest<()>> {
  if !user.can_upload() {
    return Err(BadRequest(None));
  }

  match upload_service::finalize_upload(&conn, &file_id, &request.tags, &request.source) {
    Ok(_upload) => Ok(Json(FinalizeUploadResponse {})),
    Err(_err) => Err(BadRequest(None)),
  }
}
