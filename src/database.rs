use std::env;
use std::ops::Deref;

use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection, Pool, PooledConnection};
use diesel::SqliteConnection;
use rocket::fairing::{AdHoc, Fairing};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, Rocket, State};

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

/// A pooled SQLite connection, usable as a request guard.
pub struct DatabaseConnection(PooledConnection<ConnectionManager<SqliteConnection>>);

/// Path to the SQLite database file.
pub fn database_path() -> String {
    env::var("DATABASE_PATH").unwrap_or("spin-archive.db".to_owned())
}

/// Per-connection settings. WAL + a busy timeout lets concurrent requests
/// wait on the write lock instead of failing with "database is locked".
#[derive(Debug)]
struct ConnectionOptions;

impl CustomizeConnection<SqliteConnection, r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), r2d2::Error> {
        conn.batch_execute(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(r2d2::Error::QueryError)
    }
}

impl DatabaseConnection {
    pub fn fairing() -> impl Fairing {
        AdHoc::on_attach("SQLite Pool", |rocket| {
            let manager = ConnectionManager::<SqliteConnection>::new(database_path());

            match Pool::builder()
                .max_size(8)
                .connection_customizer(Box::new(ConnectionOptions))
                .build(manager)
            {
                Ok(pool) => Ok(rocket.manage(pool)),
                Err(e) => {
                    log::error!("Failed to open database {}: {:?}", database_path(), e);
                    Err(rocket)
                }
            }
        })
    }

    pub fn get_one(rocket: &Rocket) -> Option<DatabaseConnection> {
        rocket
            .state::<SqlitePool>()
            .and_then(|pool| pool.get().ok())
            .map(DatabaseConnection)
    }
}

impl Deref for DatabaseConnection {
    type Target = SqliteConnection;

    fn deref(&self) -> &SqliteConnection {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DatabaseConnection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = match request.guard::<State<SqlitePool>>() {
            Outcome::Success(pool) => pool,
            _ => return Outcome::Failure((Status::InternalServerError, ())),
        };

        match pool.get() {
            Ok(conn) => Outcome::Success(DatabaseConnection(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

/// Returns the rowid of the most recent INSERT on this connection.
///
/// Diesel 1.4 doesn't support `RETURNING` on SQLite, so inserts read the new
/// row back by this id.
pub fn last_insert_rowid(conn: &SqliteConnection) -> QueryResult<i64> {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "last_insert_rowid()",
    ))
    .get_result(conn)
}
