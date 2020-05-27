use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::schema::tags;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, QueryableByName, Clone)]
#[table_name = "tags"]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub upload_count: i32,
}

#[derive(Debug, Insertable)]
#[table_name = "tags"]
pub struct NewTag {
    pub name: String,
}

/// Inserts a new [`Tag`] into the database.
pub fn insert(conn: &PgConnection, tag: &NewTag) -> QueryResult<usize> {
    tag.insert_into(tags::table)
        .on_conflict(tags::name)
        .do_nothing()
        .execute(conn)
}

/// Gets tags by their corresponding name.
pub fn by_names(conn: &PgConnection, tag_names: &Vec<&str>) -> Vec<Tag> {
    tags::table
        .filter(tags::name.eq_any(tag_names))
        .order((tags::name.asc(), tags::upload_count.desc()))
        .load::<Tag>(conn)
        .unwrap_or_default()
}

/// Gets tags by their corresponding name.
pub fn starting_with(conn: &PgConnection, prefix: &str, limit: i64) -> Vec<Tag> {
    tags::table
        .filter(tags::name.ilike(&format!("{}%", prefix)))
        .filter(tags::upload_count.gt(0))
        .order(tags::upload_count.desc())
        .limit(limit)
        .load::<Tag>(conn)
        .unwrap_or_default()
}
