#![allow(proc_macro_derive_resolution_fallback)]

use std::convert::TryInto;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel::{Identifiable, Queryable};
use rocket::FromForm;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[table_name = "users"]
pub struct UserModel {
  pub id: i32,
  pub username: String,
  pub password_hash: String,
  pub email: Option<String>,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
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
    Ok(NewUser {
      email: self.email,
      username: self.username,
      password_hash: hash_password(&self.password)?,
    })
  }
}

fn hash_password(password: &str) -> Result<String, RegistrationError> {
  let salt = config::secret_key();
  let config = argon2::Config::default();

  argon2::hash_encoded(password.as_ref(), salt.as_ref(), &config)
    .map_err(|e| RegistrationError::PasswordFailure)
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
) -> Result<UserModel, RegistrationError> {
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
) -> Result<UserModel, LoginError> {
  use crate::schema::users::dsl::*;

  let user: UserModel = users
    .filter(username.eq(login_username))
    .first::<UserModel>(conn)
    .map_err(|_| LoginError::InvalidPasswordOrUser)?;

  if verify_password(&user.password_hash, login_password) {
    Ok(user)
  } else {
    Err(LoginError::InvalidPasswordOrUser)
  }
}
