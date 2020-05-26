#![allow(proc_macro_derive_resolution_fallback)]

use std::convert::TryInto;
use std::io::Write;

use chrono::NaiveDateTime;
use diesel::{
    deserialize::{self, FromSql},
    expression::{helper_types::AsExprOf, AsExpression},
    prelude::*,
    serialize::{self, Output, ToSql},
    sql_types, Identifiable, PgConnection, Queryable,
};
use rocket::{
    outcome::IntoOutcome,
    request::{FromRequest, Outcome, Request},
    FromForm,
};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::database::DatabaseConnection;
use crate::schema::users;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, FromSqlRow, AsExpression)]
#[repr(i16)]
pub enum UserRole {
    Limited = 0,
    Registered = 1,
    Contributor = 2,
    Moderator = 3,
    Admin = 4,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let role = match self {
            UserRole::Limited => "limited",
            UserRole::Registered => "registered",
            UserRole::Contributor => "contributor",
            UserRole::Moderator => "moderator",
            UserRole::Admin => "admin",
        };

        write!(f, "{}", role)
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub role: UserRole,
}

impl User {
    pub fn can_upload(&self) -> bool {
        vec![UserRole::Contributor, UserRole::Moderator, UserRole::Admin].contains(&self.role)
    }

    pub fn is_contributor(&self) -> bool {
        vec![UserRole::Contributor, UserRole::Moderator, UserRole::Admin].contains(&self.role)
    }

    pub fn is_moderator(&self) -> bool {
        vec![UserRole::Moderator, UserRole::Admin].contains(&self.role)
    }

    pub fn is_admin(&self) -> bool {
        vec![UserRole::Admin].contains(&self.role)
    }
}

impl<DB> ToSql<sql_types::SmallInt, DB> for UserRole
where
    DB: diesel::backend::Backend,
    i16: ToSql<sql_types::SmallInt, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        (*self as i16).to_sql(out)
    }
}

impl<DB> FromSql<sql_types::SmallInt, DB> for UserRole
where
    DB: diesel::backend::Backend,
    i16: FromSql<sql_types::SmallInt, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i16::from_sql(bytes)? {
            0 => Ok(UserRole::Limited),
            1 => Ok(UserRole::Registered),
            2 => Ok(UserRole::Contributor),
            3 => Ok(UserRole::Moderator),
            4 => Ok(UserRole::Admin),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl AsExpression<sql_types::SmallInt> for UserRole {
    type Expression = AsExprOf<i16, sql_types::SmallInt>;

    fn as_expression(self) -> Self::Expression {
        <i16 as AsExpression<sql_types::SmallInt>>::as_expression(self as i16)
    }
}

impl AsExpression<sql_types::SmallInt> for &UserRole {
    type Expression = AsExprOf<i16, sql_types::SmallInt>;

    fn as_expression(self) -> Self::Expression {
        <i16 as AsExpression<sql_types::SmallInt>>::as_expression(*self as i16)
    }
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
    role: UserRole,
}

#[derive(Debug, FromForm)]
pub struct RegistrationFields {
    email: Option<String>,
    pub username: String,
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
            role: UserRole::Registered,
        })
    }
}

pub fn get_user_by_id(conn: &PgConnection, user_id: i32) -> Option<User> {
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

#[allow(dead_code)]
pub(crate) fn by_ids(conn: &PgConnection, ids: Vec<i32>) -> Vec<User> {
    users::table
        .filter(users::id.eq_any(ids))
        .load::<User>(conn)
        .unwrap_or_default()
}
