use diesel::{PgConnection, QueryResult};
use nanoid::nanoid;

use crate::models::api_token::{self, ApiToken, NewApiToken};
use crate::models::user::User;

pub use crate::models::api_token::{get_tokens_by_user, revoke};

/// Create a new API Token for this user.
pub fn new(conn: &PgConnection, user: &User) -> QueryResult<ApiToken> {
  let new_api_token = NewApiToken {
    token: generate_token(),
    user_id: user.id,
  };

  api_token::insert(&conn, &new_api_token)
}

fn generate_token() -> String {
  nanoid!()
}
