use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::SqliteConnection;
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

impl Tag {
    pub fn tag_url(&self) -> String {
        format!("/?q={}", self.name)
    }
}

#[derive(Debug, Insertable)]
#[table_name = "tags"]
pub struct NewTag {
    pub name: String,
}

/// Inserts a new [`Tag`] into the database.
pub fn insert(conn: &SqliteConnection, tag: &NewTag) -> QueryResult<usize> {
    diesel::insert_or_ignore_into(tags::table)
        .values(tag)
        .execute(conn)
}

/// Gets tags by their corresponding name.
pub fn by_names(conn: &SqliteConnection, tag_names: &Vec<&str>) -> Vec<Tag> {
    tags::table
        .filter(tags::name.eq_any(tag_names))
        .order((tags::name.asc(), tags::upload_count.desc()))
        .load::<Tag>(conn)
        .unwrap_or_default()
}

/// Gets tags by their corresponding name.
pub fn contains(conn: &SqliteConnection, prefix: &str, limit: i64) -> Vec<Tag> {
    tags::table
        .filter(tags::name.like(&format!("%{}%", prefix)))
        .filter(tags::upload_count.gt(0))
        .order(tags::upload_count.desc())
        .limit(limit)
        .load::<Tag>(conn)
        .unwrap_or_default()
}

/// Gets all tags.
pub fn all(conn: &SqliteConnection) -> Vec<Tag> {
    tags::table
        .filter(tags::upload_count.gt(0))
        .order((tags::name.asc(), tags::upload_count.desc()))
        .load::<Tag>(conn)
        .unwrap_or_default()
}
