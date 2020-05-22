use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::upload::Upload;
use crate::models::user::User;
use crate::schema::upload_comments;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[belongs_to(User)]
#[belongs_to(Upload)]
#[table_name = "upload_comments"]
pub struct UploadComment {
  pub id: i64,
  pub upload_id: i32,
  pub user_id: i32,
  pub comment: String,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "upload_comments"]
pub struct NewUploadComment {
  pub upload_id: i32,
  pub user_id: i32,
  pub comment: String,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "upload_comments"]
pub struct UpdateUploadComment {
  pub comment: String,
}

/// Inserts a new [`UploadComment`] into the database.
pub fn insert(
  conn: &PgConnection,
  upload_comments: &NewUploadComment,
) -> QueryResult<UploadComment> {
  upload_comments
    .insert_into(upload_comments::table)
    .get_result(conn)
}

/// Gets an [`UploadComment`] by a given `comment_id`.
pub fn get_comment_by_id(conn: &PgConnection, comment_id: i64) -> Option<UploadComment> {
  upload_comments::table
    .filter(upload_comments::id.eq(comment_id))
    .first::<UploadComment>(conn)
    .ok()
}

/// Gets all comments + authors by `upload_id`.
pub fn get_by_upload_id(
  conn: &PgConnection,
  upload_id: i32,
) -> QueryResult<Vec<(UploadComment, User)>> {
  upload_comments::table
    .inner_join(users::table)
    .filter(upload_comments::upload_id.eq(upload_id))
    .order(upload_comments::created_at.asc())
    .load::<(UploadComment, User)>(conn)
}

/// Updates a given [`UploadComment`] with new column values.
pub fn update(
  conn: &PgConnection,
  id: i64,
  comment: &UpdateUploadComment,
) -> QueryResult<UploadComment> {
  diesel::update(upload_comments::table.filter(upload_comments::id.eq(id)))
    .set(comment)
    .get_result::<UploadComment>(conn)
}
