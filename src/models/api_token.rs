use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::user::User;

use crate::schema::api_tokens;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, QueryableByName, Clone)]
#[table_name = "api_tokens"]
pub struct ApiToken {
  pub id: i64,
  pub token: String,
  pub user_id: i32,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "api_tokens"]
pub struct NewApiToken {
  pub token: String,
  pub user_id: i32,
}

/// Inserts a new [`ApiToken`] into the database.
pub fn insert(conn: &PgConnection, api_token: &NewApiToken) -> QueryResult<ApiToken> {
  api_token
    .insert_into(api_tokens::table)
    .returning(api_tokens::all_columns)
    .get_result(conn)
}

/// Deletes an [`ApiToken`] from the database.
pub fn revoke(conn: &PgConnection, user_id: i32, api_token_id: i64) -> QueryResult<usize> {
  diesel::delete(
    api_tokens::table
      .filter(api_tokens::id.eq(api_token_id))
      .filter(api_tokens::user_id.eq(user_id)),
  )
  .execute(conn)
}

/// Gets an [`ApiToken`] and corresponding [`User`] by their token string.
pub fn by_token(conn: &PgConnection, token: &str) -> Option<(ApiToken, User)> {
  use crate::schema::users;

  api_tokens::table
    .filter(api_tokens::token.eq(token))
    .inner_join(users::table)
    .first::<(ApiToken, User)>(conn)
    .ok()
}

/// Gets all tokens for a given user_id.
pub fn get_tokens_by_user(conn: &PgConnection, user_id: i32) -> Vec<ApiToken> {
  api_tokens::table
    .filter(api_tokens::user_id.eq(user_id))
    .load::<ApiToken>(conn)
    .unwrap_or_default()
}
