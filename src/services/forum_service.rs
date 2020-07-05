use diesel::connection::Connection;
use diesel::QueryResult;

use crate::models::post::{self, Post};
use crate::models::thread::{self, Thread};
use crate::DatabaseConnection;

pub fn create_thread(
    conn: &DatabaseConnection,
    author_id: i32,
    forum_id: i64,
    title: &str,
    post_content: &str,
) -> QueryResult<(Thread, Post)> {
    conn.transaction::<(Thread, Post), _, _>(|| {
        let thread = thread::insert(
            &conn,
            &thread::NewThread {
                title,
                forum_id,
                author_id,
            },
        )?;

        let post = post::insert(
            &conn,
            &post::NewPost {
                content: post_content,
                thread_id: thread.id,
                author_id,
            },
        )?;

        Ok((thread, post))
    })
}
