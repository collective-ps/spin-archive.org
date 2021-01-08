use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{sql_types, PgConnection};
use serde::{Deserialize, Serialize};

use crate::models::user::UserRole;
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

#[derive(Debug, Serialize, Deserialize, QueryableByName)]
#[table_name = "forums"]
pub struct ForumSummary {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub order_key: i32,
    pub is_open: bool,

    #[sql_type = "sql_types::Nullable<sql_types::Integer>"]
    pub post_author_id: Option<i32>,
    #[sql_type = "sql_types::Nullable<sql_types::Text>"]
    pub post_author_username: Option<String>,
    #[sql_type = "sql_types::Nullable<sql_types::SmallInt>"]
    pub post_author_role: Option<UserRole>,
    #[sql_type = "sql_types::Nullable<sql_types::Timestamptz>"]
    pub latest_post_created_at: Option<NaiveDateTime>,
    #[sql_type = "sql_types::Nullable<sql_types::Text>"]
    pub thread_title: Option<String>,
    #[sql_type = "sql_types::Nullable<sql_types::BigInt>"]
    pub thread_id: Option<i64>,
}

/// Gets forums in default order.
pub fn all(conn: &PgConnection) -> Vec<ForumSummary> {
    diesel::sql_query(
        "
            SELECT
                forums.*,
                post.created_at latest_post_created_at,
                post_author.id post_author_id,
                post_author.username post_author_username,
                post_author.role post_author_role,
                thread.title thread_title,
                thread.id thread_id
            FROM forums
            LEFT JOIN (
                SELECT MAX(posts.id) id, threads.forum_id forum_id
                FROM posts
                JOIN threads
                ON posts.thread_id = threads.id
                JOIN forums
                ON threads.forum_id = forums.id
                GROUP BY threads.forum_id
            ) latest_post ON forums.id = latest_post.forum_id
            LEFT JOIN posts post ON latest_post.forum_id = forums.id AND latest_post.id = post.id
            LEFT JOIN users as post_author ON post.author_id = post_author.id
            LEFT JOIN threads as thread ON post.thread_id = thread.id
            GROUP BY forums.id, post.created_at, latest_post.id, post_author.id, thread.title, thread.id
            ORDER BY forums.id ASC, forums.order_key DESC
        ",
    )
    .load::<ForumSummary>(conn)
    .unwrap_or_default()
}

/// Gets forum by ID.
pub fn by_id(conn: &PgConnection, forum_id: i64) -> Option<Forum> {
    forums::table
        .filter(forums::id.eq(forum_id))
        .first::<Forum>(conn)
        .ok()
}
