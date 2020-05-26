use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket::FromForm;
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::User;
use crate::services::{encoder_service, tag_service, upload_service};

/// Admin area.
#[rocket::get("/")]
pub(crate) fn index(flash: Option<FlashMessage>, user: &User) -> Result<Template, Redirect> {
    if user.is_admin() {
        let mut context = TeraContext::new();

        context::flash_context(&mut context, flash);
        context::user_context(&mut context, Some(user));

        Ok(Template::render("admin/index", &context))
    } else {
        Err(Redirect::to("/"))
    }
}

#[rocket::post("/actions/rebuild_tags")]
pub(crate) fn action_rebuild_tags(user: &User, conn: DatabaseConnection) -> Flash<Redirect> {
    if user.is_admin() {
        std::thread::spawn(move || {
            tag_service::rebuild(&conn);
        });

        Flash::success(
            Redirect::to("/admin"),
            "Started to rebuild tags. This may take a while.",
        )
    } else {
        Flash::error(Redirect::to("/"), "")
    }
}

#[rocket::post("/actions/rebuild_tag_counts")]
pub(crate) fn action_rebuild_tag_counts(user: &User, conn: DatabaseConnection) -> Flash<Redirect> {
    if user.is_admin() {
        tag_service::rebuild_tag_counts(&conn);

        Flash::success(Redirect::to("/admin"), "Rebuilt tag counts!")
    } else {
        Flash::error(Redirect::to("/"), "")
    }
}

#[derive(Serialize, Deserialize, FromForm)]
pub struct EncodeVideoRequest {
    pub file_id: String,
}

#[rocket::post("/actions/encode_video", data = "<request>")]
pub(crate) fn action_encode_video(
    user: &User,
    conn: DatabaseConnection,
    request: Form<EncodeVideoRequest>,
) -> Flash<Redirect> {
    if !user.is_admin() {
        return Flash::error(Redirect::to("/"), "");
    }

    match upload_service::get_by_file_id(&conn, &request.file_id) {
        Some(upload) => match encoder_service::enqueue_upload(&upload) {
            Ok(_) => Flash::success(Redirect::to("/"), "Sent video for encoding."),
            _ => Flash::error(Redirect::to("/"), "Could not enqueue video for encoding."),
        },
        None => return Flash::error(Redirect::to("/"), "Upload not found."),
    }
}

pub(crate) fn router() -> Vec<rocket::Route> {
    rocket::routes![
        index,
        action_rebuild_tags,
        action_rebuild_tag_counts,
        action_encode_video
    ]
}
