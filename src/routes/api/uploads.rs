use chrono::{NaiveDate, NaiveDateTime};
use rocket::response::status::BadRequest;
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};

use crate::api::{Auth, Paginated};
use crate::database::DatabaseConnection;
use crate::models;
use crate::models::upload::{FullUpload, UploadStatus};
use crate::models::user::{get_user_by_username, User};

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
