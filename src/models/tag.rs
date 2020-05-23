use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::schema::tags;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[table_name = "tags"]
pub struct Tag {
  pub id: i64,
  pub name: String,
  pub description: String,
  pub created_at: NaiveDateTime,
  pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "tags"]
pub struct NewTag {
  pub name: String,
}

/// Inserts a new [`Tag`] into the database.
pub fn insert(conn: &PgConnection, tag: &NewTag) -> QueryResult<usize> {
  tag
    .insert_into(tags::table)
    .on_conflict(tags::name)
    .do_nothing()
    .execute(conn)
}

/// Gets tags by their corresponding name.
pub fn by_names(conn: &PgConnection, tag_names: &Vec<String>) -> Vec<Tag> {
  tags::table
    .filter(tags::name.eq_any(tag_names))
    .load::<Tag>(conn)
    .unwrap_or_default()
}
