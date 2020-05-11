#![allow(proc_macro_derive_resolution_fallback)]

use std::convert::TryInto;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel::{Identifiable, Queryable};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::FromForm;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::database::DatabaseConnection;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[table_name = "users"]
pub struct User {
  pub id: i32,
  pub username: String,
  pub password_hash: String,
  pub email: Option<String>,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a User {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> Outcome<&'a User, Self::Error> {
    let user_result = request.local_cache(|| {
      let db = request.guard::<DatabaseConnection>().succeeded()?;

      request
        .cookies()
        .get_private("user_id")
        .and_then(|cookie| cookie.value().parse().ok())
        .and_then(|id| get_user_by_id(&db, id))
    });

    user_result.as_ref().or_forward(())
  }
}

#[derive(Insertable)]
#[table_name = "users"]
struct NewUser {
  email: Option<String>,
  username: String,
  password_hash: String,
}

#[derive(Debug, FromForm)]
pub struct RegistrationFields {
  email: Option<String>,
  username: String,
  password: String,
  confirm_password: String,
}

pub(crate) enum RegistrationError {
  PasswordFailure,
  AlreadyExists,
}

pub(crate) enum LoginError {
  InvalidPasswordOrUser,
}

impl TryInto<NewUser> for RegistrationFields {
  type Error = RegistrationError;

  fn try_into(self) -> Result<NewUser, Self::Error> {
    if self.password != self.confirm_password {
      return Err(RegistrationError::PasswordFailure);
    }

    Ok(NewUser {
      email: self.email,
      username: self.username,
      password_hash: hash_password(&self.password)?,
    })
  }
}

fn get_user_by_id(conn: &PgConnection, user_id: i32) -> Option<User> {
  use crate::schema::users::dsl::*;

  users.filter(id.eq(user_id)).first::<User>(conn).ok()
}

fn hash_password(password: &str) -> Result<String, RegistrationError> {
  let salt = config::secret_key();
  let config = argon2::Config::default();

  argon2::hash_encoded(password.as_ref(), salt.as_ref(), &config)
    .map_err(|_| RegistrationError::PasswordFailure)
}

fn verify_password(password: &str, hash: &str) -> bool {
  match argon2::verify_encoded(hash, password.as_ref()) {
    Ok(_) => true,
    _ => false,
  }
}

pub(crate) fn register(
  conn: &PgConnection,
  fields: RegistrationFields,
) -> Result<User, RegistrationError> {
  let new_user: NewUser = fields.try_into()?;

  diesel::insert_into(users::table)
    .values(new_user)
    .get_result(conn)
    .map_err(|_| RegistrationError::AlreadyExists)
}

pub(crate) fn login(
  conn: &PgConnection,
  login_username: &str,
  login_password: &str,
) -> Result<User, LoginError> {
  use crate::schema::users::dsl::*;

  let user: User = users
    .filter(username.eq(login_username))
    .first::<User>(conn)
    .map_err(|_| LoginError::InvalidPasswordOrUser)?;

  if verify_password(login_password, &user.password_hash) {
    Ok(user)
  } else {
    Err(LoginError::InvalidPasswordOrUser)
  }
}
