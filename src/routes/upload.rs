use std::path::Path;

use rocket::request::{FlashMessage, Form};
use rocket::response::status::BadRequest;
use rocket::response::{Flash, Redirect};
use rocket::FromForm;
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};

use crate::database::DatabaseConnection;
use crate::models::upload;
use crate::models::user::User;
use crate::s3_client::generate_signed_url;
use crate::services::{comment_service, notification_service, tag_service, upload_service};
use crate::template_utils::{BaseContext, Ructe};

#[derive(Serialize, Deserialize)]
pub struct UploadFailedResponse {
    error: &'static str,
}

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    id: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    file_name: String,
    content_length: i64,
    md5_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct FinalizeUploadResponse {}

#[derive(Serialize, Deserialize)]
pub struct FinalizeUploadRequest {
    tags: String,
    source: String,
    description: String,
    original_upload_date: Option<String>,
}

#[derive(Debug, FromForm)]
pub struct UpdateUploadRequest {
    tags: String,
    source: Option<String>,
    description: String,
    original_upload_date: Option<String>,
}

/// Upload page where a user can upload.
#[rocket::get("/upload")]
pub(crate) fn index(
    _conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: &User,
) -> Result<Ructe, Redirect> {
    if user.can_upload() {
        let ctx = BaseContext::new(Some(user), flash);

        Ok(render!(page::upload(&ctx)))
    } else {
        Err(Redirect::to("/"))
    }
}

#[rocket::get("/upload", rank = 2)]
pub(crate) fn index_not_logged_in() -> Redirect {
    Redirect::to("/")
}

/// Main [`Upload`] page.
#[rocket::get("/u/<file_id>")]
pub(crate) fn get(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    file_id: String,
) -> Result<Ructe, Redirect> {
    let ctx = BaseContext::new(user, flash);

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            upload_service::increment_view_count(&conn, upload.id.into());
            let view_count = upload_service::get_view_count(&conn, upload.id.into());
            let uploader_user = upload_service::get_uploader_user(&conn, &upload);
            let comments_with_authors = comment_service::get_comments_for_upload(&conn, &upload);
            let raw_tags = upload.tag_string.split_whitespace().collect::<Vec<&str>>();
            let tags = tag_service::by_names(&conn, &raw_tags);
            let recommended_uploads =
                upload_service::get_recommended_uploads(&conn, &tags, upload.id);

            dbg!(&recommended_uploads);

            Ok(render!(uploads::single(
                &ctx,
                &upload,
                tags,
                uploader_user,
                view_count,
                comments_with_authors,
                recommended_uploads
            )))
        }
        None => Err(Redirect::to("/404")),
    }
}

/// Embed page for an [`Upload`], primarily used for Twitter cards.
#[rocket::get("/u/<file_id>/embed")]
pub(crate) fn embed(conn: DatabaseConnection, file_id: String) -> Result<Ructe, Redirect> {
    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            upload_service::increment_view_count(&conn, upload.id.into());
            Ok(render!(uploads::embed(upload)))
        }
        None => Err(Redirect::to("/404")),
    }
}

/// Edit page for an [`Upload`].
#[rocket::get("/u/<file_id>/edit")]
pub(crate) fn edit(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: &User,
    file_id: String,
) -> Result<Ructe, Redirect> {
    let ctx = BaseContext::new(Some(user), flash);

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            if !user.can_upload() {
                return Err(Redirect::to(format!("/u/{}", upload.file_id)));
            }

            Ok(render!(uploads::edit(&ctx, upload)))
        }
        None => Err(Redirect::to("/404")),
    }
}

/// Get the audit log for a particular [`Upload`].
#[rocket::get("/u/<file_id>/log")]
pub(crate) fn log(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    file_id: String,
) -> Result<Ructe, Redirect> {
    let ctx = BaseContext::new(user, flash);

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            let view_count = upload_service::get_view_count(&conn, upload.id.into());
            let uploader_user = upload_service::get_uploader_user(&conn, &upload);
            let audit_log = upload_service::get_audit_log(&conn, &upload);

            Ok(render!(uploads::log(
                &ctx,
                upload,
                uploader_user,
                view_count,
                audit_log
            )))
        }
        None => Err(Redirect::to("/404")),
    }
}

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct NewCommentRequest {
    pub comment: String,
}

/// Comments on a given [`Upload`].
#[rocket::post("/u/<file_id>/comments", data = "<request>")]
pub(crate) fn create_comment(
    conn: DatabaseConnection,
    user: &User,
    file_id: String,
    request: Form<NewCommentRequest>,
) -> Flash<Redirect> {
    let path = format!("/u/{}", file_id);

    if request.comment.len() > 2000 {
        return Flash::error(
            Redirect::to(path),
            "Comment must be less than 2000 characters.",
        );
    }

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            match comment_service::create_comment_on_upload(&conn, &upload, &user, &request.comment)
            {
                Some(comment) => {
                    notification_service::notify_new_comment(&comment, &upload, &user);

                    Flash::success(Redirect::to(path), "Comment added!")
                }
                None => Flash::error(Redirect::to(path), "Could not create comment."),
            }
        }
        None => Flash::error(Redirect::to(path), "Could not find upload."),
    }
}

#[rocket::get("/u/<file_id>/comments/<comment_id>/edit")]
pub(crate) fn edit_comment_page(
    conn: DatabaseConnection,
    user: &User,
    file_id: String,
    comment_id: i64,
) -> Result<Ructe, Redirect> {
    let path = format!("/u/{}", file_id);

    match comment_service::get_comment_by_id(&conn, comment_id) {
        Some(comment) => {
            if comment.user_id == user.id {
                let ctx = BaseContext::new(Some(user), None);
                Ok(render!(uploads::edit_comment(&ctx, &file_id, comment)))
            } else {
                Err(Redirect::to(path))
            }
        }
        None => Err(Redirect::to(path)),
    }
}

/// Edits a comment on a given [`Upload`].
#[rocket::post("/u/<file_id>/comments/<comment_id>", data = "<request>")]
pub(crate) fn edit_comment(
    conn: DatabaseConnection,
    user: &User,
    file_id: String,
    request: Form<NewCommentRequest>,
    comment_id: i64,
) -> Flash<Redirect> {
    let path = format!("/u/{}", file_id);

    if request.comment.len() > 2000 {
        return Flash::error(
            Redirect::to(path),
            "Comment must be less than 2000 characters.",
        );
    }

    match comment_service::get_comment_by_id(&conn, comment_id) {
        Some(comment) => {
            match comment_service::edit_comment(&conn, &comment, &user, &request.comment) {
                Some(_comment) => Flash::success(Redirect::to(path), "Comment edited!"),
                None => Flash::error(Redirect::to(path), "Could not create comment."),
            }
        }
        None => Flash::error(Redirect::to(path), "Could not find comment."),
    }
}

/// Create a new upload and generate a signed URL for the user to upload directly to.
#[rocket::post("/upload", format = "json", data = "<request>")]
pub(crate) fn upload(
    conn: DatabaseConnection,
    user: &User,
    request: Json<UploadRequest>,
) -> Result<JsonValue, BadRequest<()>> {
    if !user.can_upload() {
        return Err(BadRequest(None));
    }

    let path = Path::new(&request.file_name);
    let name = path.file_name();
    let ext = path.extension();

    if name.is_none() || ext.is_none() {
        return Err(BadRequest(None));
    }

    // Basic duplicate check by MD5 hash.
    if upload_service::get_by_md5(&conn, &request.md5_hash).is_some() {
        return Ok(json!({
            "status": "error",
            "reason": "Already uploaded"
        }));
    }

    match upload_service::new_pending_upload(
        &conn,
        &user,
        name.unwrap().to_str().unwrap(),
        ext.unwrap().to_str().unwrap(),
        request.content_length,
        Some(request.md5_hash.clone()),
    ) {
        Ok(upload) => {
            let file_name = format!("{}.{}", &upload.file_id, &upload.file_ext);

            Ok(json!({
                "id": upload.file_id.to_owned(),
                "url": generate_signed_url("uploads", &file_name),
            }))
        }
        Err(_) => Ok(json!({
            "status": "error",
            "reason": "Server error"
        })),
    }
}

/// Finalizes an upload and starts processing it.
#[rocket::post("/upload/<file_id>/finalize", format = "json", data = "<request>")]
pub(crate) fn finalize(
    conn: DatabaseConnection,
    user: &User,
    file_id: String,
    request: Json<FinalizeUploadRequest>,
) -> Result<Json<FinalizeUploadResponse>, BadRequest<()>> {
    if !user.can_upload() {
        return Err(BadRequest(None));
    }

    let parsed_original_date = match &request.original_upload_date {
        Some(date) if date.is_empty() => Ok(None),
        Some(date) => {
            chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").map(|result| Some(result))
        }
        _ => Ok(None),
    };

    if let Err(_) = parsed_original_date {
        return Err(BadRequest(None));
    }

    match upload_service::finalize_upload(
        &conn,
        &user,
        &file_id,
        &request.tags,
        &request.source,
        &request.description,
        parsed_original_date.unwrap(),
    ) {
        Ok(_upload) => Ok(Json(FinalizeUploadResponse {})),
        Err(_err) => Err(BadRequest(None)),
    }
}

/// Updates a given upload `file_id` with new params.
#[rocket::post("/upload/<file_id>", data = "<request>")]
pub(crate) fn update(
    conn: DatabaseConnection,
    user: &User,
    file_id: String,
    request: Form<UpdateUploadRequest>,
) -> Flash<Redirect> {
    let path = format!("/u/{}", file_id);

    if !user.can_upload() {
        return Flash::error(Redirect::to(path), "You can't do that.");
    }

    let source = request.source.as_ref().unwrap_or(&"".to_string()).clone();
    let parsed_original_date = match &request.original_upload_date {
        Some(date) if date.is_empty() => Ok(None),
        Some(date) => {
            chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").map(|result| Some(result))
        }
        _ => Ok(None),
    };

    if let Err(_) = parsed_original_date {
        return Flash::error(Redirect::to(path), "Invalid original upload date");
    }

    match upload_service::update_upload(
        &conn,
        user.id,
        &file_id,
        &request.tags,
        &source,
        &request.description,
        parsed_original_date.unwrap(),
    ) {
        Ok(_upload) => Flash::success(Redirect::to(path), "Edited!"),
        Err(_err) => Flash::error(
            Redirect::to(format!("{}/edit", path)),
            "Could not edit upload.",
        ),
    }
}

/// Marks a given upload as Deleted.
#[rocket::post("/upload/<file_id>/delete")]
pub(crate) fn delete(conn: DatabaseConnection, user: &User, file_id: String) -> Flash<Redirect> {
    let path = format!("/u/{}", file_id);

    if !user.is_moderator() {
        return Flash::error(Redirect::to(path), "You do not have access to do that.");
    }

    match upload_service::get_by_file_id(&conn, &file_id)
        .ok_or("No upload found.")
        .and_then(|upload| {
            upload_service::delete(&conn, &upload, &user).map_err(|_| "Could not delete upload.")
        })
        .and_then(|_| {
            Ok(Flash::success(
                Redirect::to(path.clone()),
                "Marked upload as deleted.",
            ))
        }) {
        Ok(result) => result,
        Err(error) => Flash::error(Redirect::to(path.clone()), error),
    }
}

#[rocket::get("/random")]
pub(crate) fn random(conn: DatabaseConnection) -> Redirect {
    match upload_service::random(&conn) {
        Some(upload) => {
            let path = format!("/u/{}", upload.file_id);
            Redirect::to(path)
        }
        None => Redirect::to("/404"),
    }
}
