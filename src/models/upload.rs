use std::io::Write;

use chrono::NaiveDateTime;
use diesel::{
  deserialize::{self, FromSql},
  expression::{helper_types::AsExprOf, AsExpression},
  prelude::*,
  serialize::{self, Output, ToSql},
  sql_types, AsChangeset, Identifiable, PgConnection, Queryable,
};
use serde::{Deserialize, Serialize};

use crate::pagination::*;
use crate::schema::uploads;

type AllColumns = (
  uploads::id,
  uploads::status,
  uploads::file_id,
  uploads::file_size,
  uploads::file_name,
  uploads::md5_hash,
  uploads::uploader_user_id,
  uploads::source,
  uploads::created_at,
  uploads::updated_at,
  uploads::file_ext,
  uploads::tag_string,
);

pub const ALL_COLUMNS: AllColumns = (
  uploads::id,
  uploads::status,
  uploads::file_id,
  uploads::file_size,
  uploads::file_name,
  uploads::md5_hash,
  uploads::uploader_user_id,
  uploads::source,
  uploads::created_at,
  uploads::updated_at,
  uploads::file_ext,
  uploads::tag_string,
);

type All = diesel::dsl::Select<uploads::table, AllColumns>;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, FromSqlRow, AsExpression)]
#[repr(i16)]
pub enum UploadStatus {
  Pending = 0,
  Processing = 1,
  Completed = 2,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, AsChangeset)]
#[table_name = "uploads"]
pub struct Upload {
  pub id: i32,
  pub status: UploadStatus,
  pub file_id: String,
  pub file_size: Option<i64>,
  pub file_name: Option<String>,
  pub md5_hash: Option<String>,
  pub uploader_user_id: Option<i32>,
  pub source: Option<String>,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
  pub file_ext: String,
  pub tag_string: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, AsChangeset)]
#[table_name = "uploads"]
pub struct UpdateUpload {
  pub id: i32,
  pub status: UploadStatus,
  pub source: Option<String>,
  pub tag_string: String,
}

const ASSET_HOST: &'static str = "https://bits.spin-archive.org/uploads";

impl Upload {
  /// Gets a full URL to the thumbnail for this upload. (may not actually exist!)
  pub fn get_thumbnail_url(&self) -> String {
    format!(
      "{host}/{file_id}.jpg",
      host = ASSET_HOST,
      file_id = self.file_id
    )
  }

  /// Gets the full URL to where the file is stored.
  pub fn get_file_url(&self) -> String {
    format!(
      "{host}/{file_id}.{ext}",
      host = ASSET_HOST,
      file_id = self.file_id,
      ext = self.file_ext
    )
  }
}

impl<DB> ToSql<sql_types::SmallInt, DB> for UploadStatus
where
  DB: diesel::backend::Backend,
  i16: ToSql<sql_types::SmallInt, DB>,
{
  fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
    (*self as i16).to_sql(out)
  }
}

impl<DB> FromSql<sql_types::SmallInt, DB> for UploadStatus
where
  DB: diesel::backend::Backend,
  i16: FromSql<sql_types::SmallInt, DB>,
{
  fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
    match i16::from_sql(bytes)? {
      0 => Ok(UploadStatus::Pending),
      1 => Ok(UploadStatus::Processing),
      2 => Ok(UploadStatus::Completed),
      _ => Err("Unrecognized enum variant".into()),
    }
  }
}

impl AsExpression<sql_types::SmallInt> for UploadStatus {
  type Expression = AsExprOf<i16, sql_types::SmallInt>;

  fn as_expression(self) -> Self::Expression {
    <i16 as AsExpression<sql_types::SmallInt>>::as_expression(self as i16)
  }
}

impl AsExpression<sql_types::SmallInt> for &UploadStatus {
  type Expression = AsExprOf<i16, sql_types::SmallInt>;

  fn as_expression(self) -> Self::Expression {
    <i16 as AsExpression<sql_types::SmallInt>>::as_expression(*self as i16)
  }
}

#[derive(Insertable)]
#[table_name = "uploads"]
pub struct PendingUpload {
  pub status: UploadStatus,
  pub file_id: String,
  pub uploader_user_id: i32,
  pub file_name: String,
  pub file_ext: String,
}

#[derive(Insertable)]
#[table_name = "uploads"]
pub struct FinalizeUpload {
  pub status: UploadStatus,
  pub tag_string: String,
}

/// Gets an [`Upload`] by `file_id`.
pub fn get_by_file_id(conn: &PgConnection, search_file_id: &str) -> Option<Upload> {
  use crate::schema::uploads::dsl::*;

  uploads
    .filter(file_id.eq(search_file_id))
    .select(ALL_COLUMNS)
    .first::<Upload>(conn)
    .ok()
}

/// Updates a given [`Upload`] with new column values.
pub fn update(conn: &PgConnection, upload: &UpdateUpload) -> QueryResult<Upload> {
  diesel::update(uploads::table)
    .set(upload)
    .returning(ALL_COLUMNS)
    .get_result::<Upload>(conn)
}

/// Inserts a given [`PendingUpload`] into the database.
pub fn insert_pending_upload(
  conn: &PgConnection,
  pending_upload: &PendingUpload,
) -> QueryResult<Upload> {
  diesel::insert_into(uploads::table)
    .values(pending_upload)
    .returning(ALL_COLUMNS)
    .get_result(conn)
}

/// Index query for uploads, fetches completed uploads by the page number provided.
///
/// Returns a tuple: (Vec<Upload>, page_count).
pub fn index(conn: &PgConnection, page: i64) -> QueryResult<(Vec<Upload>, i64)> {
  let query = uploads::table
    .order(uploads::updated_at.desc())
    .filter(uploads::status.eq(UploadStatus::Completed))
    .select(ALL_COLUMNS)
    .paginate(page)
    .per_page(25);

  query.load_and_count_pages::<Upload>(&conn)
}
