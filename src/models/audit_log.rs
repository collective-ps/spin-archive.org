use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::upload::{Upload, ALL_COLUMNS as ALL_UPLOAD_COLUMNS};
use crate::models::user::User;
use crate::schema::{audit_log, uploads, users};

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

pub fn get_paginated_log(
    conn: &PgConnection,
    page: i64,
    per_page: i64,
) -> Vec<(AuditLog, User, Upload)> {
    audit_log::table
        .filter(audit_log::table_name.eq("uploads"))
        .inner_join(uploads::table.on(uploads::id.eq(audit_log::row_id)))
        .inner_join(users::table)
        .select((
            audit_log::all_columns,
            users::all_columns,
            ALL_UPLOAD_COLUMNS,
        ))
        .order_by(audit_log::changed_date.desc())
        .limit(per_page)
        .offset((page - 1) * per_page)
        .load::<(AuditLog, User, Upload)>(conn)
        .unwrap_or_default()
}

pub fn get_log_count(conn: &PgConnection) -> i64 {
    use diesel::dsl::count;

    audit_log::table
        .select(count(audit_log::id))
        .filter(audit_log::table_name.eq("uploads"))
        .first::<i64>(conn)
        .unwrap_or_default()
}
