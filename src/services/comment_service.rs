use diesel::PgConnection;

use crate::models::upload::Upload;
use crate::models::upload_comment::{self, NewUploadComment, UpdateUploadComment, UploadComment};
use crate::models::user::User;

pub use crate::models::upload_comment::{
    get_comment_by_id, get_comment_count_by_user_id, get_paginated_comments, get_recent_comments,
};

pub fn create_comment_on_upload(
    conn: &PgConnection,
    upload: &Upload,
    user: &User,
    comment: &str,
) -> Option<UploadComment> {
    let comment = NewUploadComment {
        upload_id: upload.id,
        user_id: user.id,
        comment: comment.to_string(),
    };

    upload_comment::insert(conn, &comment).ok()
}

pub fn edit_comment(
    conn: &PgConnection,
    upload_comment: &UploadComment,
    user: &User,
    comment: &str,
) -> Option<UploadComment> {
    if upload_comment.user_id != user.id {
        return None;
    }

    let update = UpdateUploadComment {
        comment: comment.to_string(),
    };

    upload_comment::update(&conn, upload_comment.id, &update).ok()
}

pub fn get_comments_for_upload(conn: &PgConnection, upload: &Upload) -> Vec<(UploadComment, User)> {
    upload_comment::get_by_upload_id(&conn, upload.id).unwrap_or_default()
}
