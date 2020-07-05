use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::schema::forums;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, QueryableByName, Clone)]
#[table_name = "forums"]
pub struct Forum {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub order_key: i32,
    pub is_open: bool,
}

/// Gets forums in default order.
pub fn all(conn: &PgConnection) -> Vec<Forum> {
    forums::table
        .order((forums::id.asc(), forums::order_key.asc()))
        .load::<Forum>(conn)
        .unwrap_or_default()
}

/// Gets forum by ID.
pub fn by_id(conn: &PgConnection, forum_id: i64) -> Option<Forum> {
    forums::table
        .filter(forums::id.eq(forum_id))
        .first::<Forum>(conn)
        .ok()
}
