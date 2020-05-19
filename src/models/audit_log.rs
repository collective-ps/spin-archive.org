use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::user::User;
use crate::schema::audit_log;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[table_name = "audit_log"]
pub struct AuditLog {
  pub id: i64,
  pub table_name: String,
  pub column_name: String,
  pub row_id: i32,
  pub changed_date: NaiveDateTime,

  /// User ID who changed this.
  pub changed_by: i32,

  pub old_value: String,
  pub new_value: String,
}

#[derive(Debug, Insertable)]
#[table_name = "audit_log"]
pub struct NewAuditLog {
  pub table_name: String,
  pub column_name: String,
  pub row_id: i32,
  pub changed_date: NaiveDateTime,
  pub changed_by: i32,
  pub old_value: String,
  pub new_value: String,
}

pub fn insert(conn: &PgConnection, audit_log: &NewAuditLog) -> QueryResult<AuditLog> {
  audit_log.insert_into(audit_log::table).get_result(conn)
}

pub fn get_by_row_id(
  conn: &PgConnection,
  table_name: &str,
  row_id: i32,
) -> QueryResult<Vec<(AuditLog, User)>> {
  audit_log::table
    .inner_join(users::table)
    .filter(audit_log::table_name.eq(table_name))
    .filter(audit_log::row_id.eq(row_id))
    .load::<(AuditLog, User)>(conn)
}
