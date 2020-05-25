use std::path::Path;

use rocket::request::{FlashMessage, Form};
use rocket::response::status::BadRequest;
use rocket::response::{Flash, Redirect};
use rocket::FromForm;
use rocket_contrib::json::Json;
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::upload;
use crate::models::user::User;
use crate::s3_client::generate_signed_url;
use crate::services::{comment_service, tag_service, upload_service};

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    id: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    file_name: String,
    content_length: i32,
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
pub(crate) fn index(flash: Option<FlashMessage>, user: &User) -> Result<Template, Redirect> {
    if user.can_upload() {
        let mut context = TeraContext::new();

        context::flash_context(&mut context, flash);
        context::user_context(&mut context, Some(user));

        Ok(Template::render("upload", &context))
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
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            upload_service::increment_view_count(&conn, upload.id.into());
            let view_count = upload_service::get_view_count(&conn, upload.id.into());
            let uploader_user = upload_service::get_uploader_user(&conn, &upload);
            let comments_with_authors = comment_service::get_comments_for_upload(&conn, &upload);
            let raw_tags = upload.tag_string.split_whitespace().collect::<Vec<&str>>();
            let tags = tag_service::by_names(&conn, &raw_tags);

            context.insert("upload", &upload);
            context.insert("view_count", &view_count);
            context.insert("uploader", &uploader_user);
            context.insert("comments_with_authors", &comments_with_authors);
            context.insert("tags", &tags);

            Ok(Template::render("uploads/single", &context))
        }
        None => Err(Redirect::to("/404")),
    }
}

/// Embed page for an [`Upload`], primarily used for Twitter cards.
#[rocket::get("/u/<file_id>/embed")]
pub(crate) fn embed(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    file_id: String,
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            upload_service::increment_view_count(&conn, upload.id.into());
            let view_count = upload_service::get_view_count(&conn, upload.id.into());
            let uploader_user = upload_service::get_uploader_user(&conn, &upload);

            context.insert("upload", &upload);
            context.insert("view_count", &view_count);
            context.insert("uploader", &uploader_user);

            Ok(Template::render("uploads/embed", &context))
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
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, Some(user));

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            if !user.can_upload() {
                return Err(Redirect::to(format!("/u/{}", upload.file_id)));
            }

            let view_count = upload_service::get_view_count(&conn, upload.id.into());
            let uploader_user = upload_service::get_uploader_user(&conn, &upload);

            context.insert("upload", &upload);
            context.insert("view_count", &view_count);
            context.insert("uploader", &uploader_user);

            Ok(Template::render("uploads/edit", &context))
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
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    match upload::get_by_file_id(&conn, &file_id) {
        Some(upload) => {
            let view_count = upload_service::get_view_count(&conn, upload.id.into());
            let uploader_user = upload_service::get_uploader_user(&conn, &upload);
            let audit_log = upload_service::get_audit_log(&conn, &upload);

            context.insert("upload", &upload);
            context.insert("view_count", &view_count);
            context.insert("uploader", &uploader_user);
            context.insert("audit_log", &audit_log);

            Ok(Template::render("uploads/log", &context))
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
                Some(_comment) => Flash::success(Redirect::to(path), "Comment added!"),
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
) -> Result<Template, Redirect> {
    let path = format!("/u/{}", file_id);
    let mut context = TeraContext::new();

    context::user_context(&mut context, Some(user));

    match comment_service::get_comment_by_id(&conn, comment_id) {
        Some(comment) => {
            if comment.user_id == user.id {
                context.insert("comment", &comment);
                context.insert("file_id", &file_id);

                Ok(Template::render("uploads/edit_comment", &context))
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
) -> Result<Json<UploadResponse>, BadRequest<()>> {
    if !user.can_upload() {
        return Err(BadRequest(None));
    }

    let path = Path::new(&request.file_name);
    let name = path.file_name();
    let ext = path.extension();

    if name.is_none() || ext.is_none() {
        return Err(BadRequest(None));
    }

    match upload_service::new_pending_upload(
        &conn,
        &user,
        name.unwrap().to_str().unwrap(),
        ext.unwrap().to_str().unwrap(),
    ) {
        Ok(upload) => {
            let file_name = format!("{}.{}", &upload.file_id, &upload.file_ext);

            Ok(Json(UploadResponse {
                id: upload.file_id.to_owned(),
                url: generate_signed_url("uploads", &file_name),
            }))
        }
        Err(_) => Err(BadRequest(None)),
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
        return Flash::error(Redirect::to(path), "");
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
