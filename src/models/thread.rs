use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{sql_types, PgConnection};
use serde::{Deserialize, Serialize};

use crate::models::user::{User, UserRole};
use crate::schema::threads;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, QueryableByName, Clone)]
#[table_name = "threads"]
pub struct Thread {
    pub id: i64,
    pub title: String,
    pub forum_id: i64,
    pub author_id: i32,
    pub is_sticky: bool,
    pub is_open: bool,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, QueryableByName)]
#[table_name = "threads"]
pub struct ThreadSummary {
    pub id: i64,
    pub title: String,
    pub forum_id: i64,
    pub author_id: i32,
    pub is_sticky: bool,
    pub is_open: bool,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    #[sql_type = "sql_types::Integer"]
    pub thread_author_id: i32,
    #[sql_type = "sql_types::Text"]
    pub thread_author_username: String,
    #[sql_type = "sql_types::SmallInt"]
    pub thread_author_role: UserRole,
    #[sql_type = "sql_types::Integer"]
    pub post_author_id: i32,
    #[sql_type = "sql_types::Text"]
    pub post_author_username: String,
    #[sql_type = "sql_types::SmallInt"]
    pub post_author_role: UserRole,
    #[sql_type = "sql_types::Timestamptz"]
    pub latest_post_created_at: NaiveDateTime,
    #[sql_type = "sql_types::BigInt"]
    pub post_count: i64,
}

#[derive(Debug, Insertable)]
#[table_name = "threads"]
pub struct NewThread<'a> {
    pub title: &'a str,
    pub forum_id: i64,
    pub author_id: i32,
}

/// Gets thread by id.
pub fn by_id(conn: &PgConnection, thread_id: i64) -> Option<(Thread, User)> {
    threads::table
        .filter(threads::id.eq(thread_id))
        .inner_join(users::table)
        .select((threads::all_columns, users::all_columns))
        .first::<(Thread, User)>(conn)
        .ok()
}

pub fn set_sticky(conn: &PgConnection, thread_id: i64, is_sticky: bool) -> QueryResult<Thread> {
    diesel::update(threads::table.filter(threads::id.eq(thread_id)))
    .set(threads::is_sticky.eq(is_sticky))
    .get_result::<Thread>(conn)
}

/// Gets threads in default order, by forum_id.
pub fn by_forum_id(conn: &PgConnection, forum_id: i64) -> Vec<ThreadSummary> {
    use diesel::sql_types::*;

    diesel::sql_query(
        "
            SELECT
                threads.*,
                thread_author.id thread_author_id,
                thread_author.username thread_author_username,
                thread_author.role thread_author_role,
                post.created_at latest_post_created_at,
                post_author.id post_author_id,
                post_author.username post_author_username,
                post_author.role post_author_role,
                count(posts.id) post_count
            FROM threads
            INNER JOIN users as thread_author
            ON thread_author.id = threads.author_id
            JOIN (
                SELECT MAX(posts.id) id, posts.thread_id
                FROM posts
                JOIN threads
                ON posts.thread_id = threads.id
                GROUP BY posts.thread_id
            ) latest_post
            ON threads.id = latest_post.thread_id
            JOIN posts post
            ON latest_post.thread_id = post.thread_id AND latest_post.id = post.id
            INNER JOIN users as post_author ON post.author_id = post_author.id
            INNER JOIN posts ON posts.thread_id = threads.id
            WHERE threads.forum_id = $1 AND threads.is_deleted = false
            GROUP BY threads.id, thread_author.id, posts.thread_id, post.created_at, latest_post.id, post_author.id
            ORDER BY threads.is_sticky DESC, post.created_at DESC
        ",
    )
    .bind::<BigInt, _>(forum_id)
    .get_results(conn)
    .unwrap_or_default()
}

pub fn insert(conn: &PgConnection, thread: &NewThread) -> QueryResult<Thread> {
    thread
        .insert_into(threads::table)
        .returning(threads::all_columns)
        .get_result(conn)
}
