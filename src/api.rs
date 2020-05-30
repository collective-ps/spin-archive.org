use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::Serialize;

use crate::database::DatabaseConnection;
use crate::models::api_token;
use crate::models::{api_token::ApiToken, user::User};

#[derive(Serialize)]
pub struct Paginated<T: Serialize> {
  pub data: Vec<T>,
  pub page_size: i64,
  pub page_count: i64,
  pub total_count: i64,
}

pub struct Auth<'a> {
  pub api_token: &'a ApiToken,
  pub user: &'a User,
}

impl<'a, 'r> FromRequest<'a, 'r> for Auth<'a> {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> Outcome<Auth<'a>, Self::Error> {
    let api_result = request
      .local_cache(|| {
        let db = request.guard::<DatabaseConnection>().succeeded()?;

        request
          .headers()
          .get_one("Authorization")
          .and_then(|value| {
            if value.starts_with("Bearer ") {
              let token = &value[7..];
              api_token::by_token(&db, &token)
            } else {
              None
            }
          })
      })
      .as_ref();

    if let Some((api_token, user)) = api_result {
      let auth = Auth { api_token, user };
      Outcome::Success(auth)
    } else {
      Outcome::Failure((Status::Forbidden, ()))
    }
  }
}
