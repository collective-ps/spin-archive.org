use diesel::prelude::*;
use diesel::PgConnection;
use nanoid::nanoid;

use crate::models::upload::{PendingUpload, Upload, UploadStatus};
use crate::schema::uploads;

#[allow(dead_code)]
pub(crate) enum UploadError {
  AlreadyExists,
  DatabaseError,
}

/// Creates a new pending upload.
pub(crate) fn new_pending_upload(conn: &PgConnection) -> Result<Upload, UploadError> {
  let pending_upload = PendingUpload {
    status: UploadStatus::Pending,
    file_id: nanoid!(),
  };

  diesel::insert_into(uploads::table)
    .values(pending_upload)
    .get_result(conn)
    .map_err(|_| UploadError::DatabaseError)
}
