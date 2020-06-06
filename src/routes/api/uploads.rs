use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use log::warn;
use rocket::response::status::BadRequest;
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};

use crate::api::{Auth, Paginated};
use crate::database::DatabaseConnection;
use crate::ingestors;
use crate::models;
use crate::models::upload::FullUpload;
use crate::models::user::{get_user_by_username, User};
use crate::s3_client::generate_signed_url;
use crate::services::upload_service::{self, UploadError};

#[derive(Serialize)]
pub struct FullUploadJson {
  id: String,
  status: String,
  file_name: String,
  uploader_username: String,
  comment_count: i64,
  view_count: i64,
  url: String,
  source: Option<String>,
  original_upload_date: Option<NaiveDate>,
  description: String,
  created_at: NaiveDateTime,
  updated_at: NaiveDateTime,
}

impl From<&FullUpload> for FullUploadJson {
  fn from(upload: &FullUpload) -> Self {
    FullUploadJson {
      id: upload.file_id.clone(),
      status: upload.status.to_string().to_lowercase(),
      file_name: upload.file_name.clone().unwrap_or_default(),
      uploader_username: upload.uploader_username.clone(),
      comment_count: upload.comment_count,
      view_count: upload.view_count,
      url: format!("https://spin-archive.org/u/{}", &upload.file_id),
      source: upload.source.clone(),
      original_upload_date: upload.original_upload_date,
      description: upload.description.clone(),
      created_at: upload.created_at,
      updated_at: upload.updated_at,
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct ValidateChecksumRequest {
  checksums: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ValidateChecksumResponse {
  checksums: Vec<String>,
}

#[rocket::post("/uploads/checksum", format = "json", data = "<request>")]
pub fn validate_checksum(
  conn: DatabaseConnection,
  request: Json<ValidateChecksumRequest>,
  _auth: Auth,
) -> Json<ValidateChecksumResponse> {
  let uploads = upload_service::where_md5(&conn, &request.checksums);
  let found_checksums = uploads
    .into_iter()
    .map(|upload| upload.md5_hash.unwrap())
    .collect();
  let response = ValidateChecksumResponse {
    checksums: found_checksums,
  };

  Json(response)
}

#[derive(Serialize, Deserialize)]
pub struct SearchParams {
  page: Option<i64>,
  query: Option<String>,
}

/// Upload page where a user can upload.
#[rocket::post("/uploads/search", format = "json", data = "<request>")]
pub fn search(
  conn: DatabaseConnection,
  request: Json<SearchParams>,
  _auth: Auth,
) -> Json<Paginated<FullUploadJson>> {
  let per_page = 50;
  let current_page = request.page.unwrap_or(1);
  let mut query = request.query.clone().unwrap_or_default();

  // Check if the query has an `uploader:[USERNAME]` tag.
  let mut uploader: Option<User> = None;

  if !query.is_empty() {
    let uploader_regex = regex::Regex::new(r"(uploader:)([a-z_A-Z\d]*)\s?").unwrap();

    match uploader_regex.captures(&query) {
      None => (),
      Some(matches) => {
        let full_match = &matches[0];
        let username = &matches[2];

        match get_user_by_username(&conn, &username) {
          Some(user) => uploader = Some(user),
          _ => (),
        }

        query = query.replace(full_match, "");
      }
    }
  }

  let (uploads, page_count, total_count) =
    models::upload::index(&conn, current_page, per_page, &query, uploader);

  let full_uploads = uploads
    .iter()
    .map(|upload| upload.into())
    .collect::<Vec<FullUploadJson>>();

  let response = Paginated {
    data: full_uploads,
    page_size: per_page,
    page_count,
    total_count,
  };

  Json(response)
}

#[derive(Serialize, Deserialize)]
pub struct NewUploadRequest {
  file_name: String,
  content_length: i64,
}

#[derive(Serialize, Deserialize)]
pub struct NewUploadResponse {
  id: String,
  url: String,
}

#[rocket::post("/uploads", format = "json", data = "<request>")]
pub fn new(
  conn: DatabaseConnection,
  request: Json<NewUploadRequest>,
  auth: Auth,
) -> Result<Json<NewUploadResponse>, BadRequest<JsonValue>> {
  let user = auth.user;

  if !user.can_upload() {
    return Err(BadRequest(Some(json!({
      "status": "no_permission",
      "reason": "You do not have permission to upload."
    }))));
  }

  let path = Path::new(&request.file_name);
  let name = path.file_name();
  let ext = path.extension();

  if name.is_none() || ext.is_none() {
    return Err(BadRequest(Some(json!({
        "status": "invalid_file_name",
        "reason": "Invalid file_name provided."
    }))));
  }

  // Basic duplicate check by file name / ext / file size.
  if upload_service::get_by_original_file(
    &conn,
    name.unwrap().to_str().unwrap(),
    ext.unwrap().to_str().unwrap(),
    request.content_length,
  )
  .is_some()
  {
    return Err(BadRequest(Some(json!({
        "status": "already_uploaded",
        "reason": "Already uploaded"
    }))));
  }

  match upload_service::new_pending_upload(
    &conn,
    &user,
    name.unwrap().to_str().unwrap(),
    ext.unwrap().to_str().unwrap(),
    request.content_length,
    None,
  ) {
    Ok(upload) => {
      let file_name = format!("{}.{}", &upload.file_id, &upload.file_ext);

      Ok(Json(NewUploadResponse {
        id: upload.file_id.to_owned(),
        url: generate_signed_url("uploads", &file_name),
      }))
    }
    Err(UploadError::UploadLimitReached) => Err(BadRequest(Some(json!({
        "status": "upload_limit_reached",
        "reason": "Upload limit reached."
    })))),
    Err(_) => Err(BadRequest(Some(json!({
        "status": "error",
        "reason": "Server error"
    })))),
  }
}

#[derive(Serialize, Deserialize)]
pub struct FinalizeUploadResponse {
  id: String,
  url: String,
}

#[derive(Serialize, Deserialize)]
pub struct FinalizeUploadRequest {
  id: String,
  tags: String,
  source: String,
  description: String,
  original_upload_date: Option<String>,
}

/// Finalizes an upload and starts processing it.
#[rocket::post("/uploads/finalize", format = "json", data = "<request>")]
pub(crate) fn finalize(
  conn: DatabaseConnection,
  auth: Auth,
  request: Json<FinalizeUploadRequest>,
) -> Result<Json<FinalizeUploadResponse>, BadRequest<JsonValue>> {
  let user = auth.user;

  if !user.can_upload() {
    return Err(BadRequest(Some(json!({
      "status": "no_permission",
      "reason": "You do not have permission to upload."
    }))));
  }

  let parsed_original_date = match &request.original_upload_date {
    Some(date) if date.is_empty() => Ok(None),
    Some(date) => chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").map(|result| Some(result)),
    _ => Ok(None),
  };

  if let Err(_) = parsed_original_date {
    return Err(BadRequest(Some(json!({
        "status": "invalid_original_upload_date",
        "reason": "Invalid original_upload_date provided."
    }))));
  }

  match upload_service::finalize_upload(
    &conn,
    &user,
    &request.id,
    &request.tags,
    &request.source,
    &request.description,
    parsed_original_date.unwrap(),
  ) {
    Ok(upload) => Ok(Json(FinalizeUploadResponse {
      id: upload.file_id.clone(),
      url: format!("https://spin-archive.org/u/{}", upload.file_id),
    })),
    Err(_) => Err(BadRequest(Some(json!({
        "status": "error",
        "reason": "Server error"
    })))),
  }
}

#[derive(Deserialize)]
pub struct TwitterUpload {
  url: String,
  tags: String,
}

#[derive(Serialize)]
pub struct TwitterUploadResponse {
  id: String,
  url: String,
}

#[rocket::post("/uploads/twitter", format = "json", data = "<request>")]
pub fn twitter(
  conn: DatabaseConnection,
  auth: Option<Auth>,
  user: Option<&User>,
  request: Json<TwitterUpload>,
) -> Result<Json<TwitterUploadResponse>, BadRequest<JsonValue>> {
  if auth.is_none() && user.is_none() {
    return Err(BadRequest(Some(json!({
        "status": "no_permissions",
        "reason": "Unauthorized"
    }))));
  }

  if auth.is_none() && !user.unwrap().is_contributor() {
    return Err(BadRequest(Some(json!({
        "status": "no_permissions",
        "reason": "Unauthorized"
    }))));
  }

  let uploader = match auth {
    None => user.unwrap(),
    Some(auth) => auth.user,
  };

  let existing_upload = upload_service::get_by_source(&conn, &request.url);

  if existing_upload.is_some() {
    return Err(BadRequest(Some(json!({
        "status": "already_exists",
        "reason": "An upload with this URL already exists"
    }))));
  }

  match ingestors::twitter::download_from_tweet(conn, &uploader, &request.url, &request.tags) {
    Ok(upload) => Ok(Json(TwitterUploadResponse {
      id: upload.file_id.clone(),
      url: format!("https://spin-archive.org/u/{}", upload.file_id),
    })),
    Err(err) => {
      warn!("[api/v1/uploads/twitter] {}", err);
      Err(BadRequest(Some(json!({
          "status": "error",
          "reason": format!("{}", err)
      }))))
    }
  }
}
