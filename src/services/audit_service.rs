use chrono::Utc;
use diesel::PgConnection;

use crate::models::audit_log::{self, AuditLog, NewAuditLog};

pub fn create_audit_log(
    conn: &PgConnection,
    table_name: &str,
    column_name: &str,
    row_id: i32,
    changed_by: i32,
    old_value: &str,
    new_value: &str,
) -> Option<AuditLog> {
    if !old_value.eq_ignore_ascii_case(new_value) {
        let audit_log = NewAuditLog {
            table_name: table_name.to_string(),
            column_name: column_name.to_string(),
            row_id,
            changed_date: Utc::now().naive_utc(),
            changed_by,
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        };

        audit_log::insert(conn, &audit_log).ok()
    } else {
        None
    }
}
