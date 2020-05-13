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

use crate::schema::uploads;

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
pub(crate) struct PendingUpload {
  pub status: UploadStatus,
  pub file_id: String,
  pub uploader_user_id: i32,
}

/// Gets an [`Upload`] by `file_id`.
pub fn get_by_file_id(conn: &PgConnection, search_file_id: &str) -> Option<Upload> {
  use crate::schema::uploads::dsl::*;

  uploads
    .filter(file_id.eq(search_file_id))
    .first::<Upload>(conn)
    .ok()
}

/// Updates a given [`Upload`] with new column values.
pub fn update(conn: &PgConnection, upload: &Upload) -> QueryResult<Upload> {
  diesel::update(uploads::table)
    .set(upload)
    .get_result::<Upload>(conn)
}
