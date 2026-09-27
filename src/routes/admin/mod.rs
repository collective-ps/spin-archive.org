use log::warn;
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket::FromForm;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseConnection;
use crate::models::upload;
use crate::models::user::User;
use crate::s3_client;
use crate::services::{encoder_service, tag_service, upload_service};
use crate::template_utils::{BaseContext, Ructe};

/// Admin area.
#[rocket::get("/")]
pub(crate) fn index(flash: Option<FlashMessage>, user: &User) -> Result<Ructe, Redirect> {
    if user.is_admin() {
        let ctx = BaseContext::new(Some(user), flash).noindex_nofollow();

        Ok(render!(admin::index(&ctx)))
    } else {
        Err(Redirect::to("/"))
    }
}

/// Encoding queue and encoder Machines.
#[rocket::get("/encoding")]
pub(crate) fn encoding(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: &User,
) -> Result<Ructe, Redirect> {
    if !user.is_admin() {
        return Err(Redirect::to("/"));
    }

    let ctx = BaseContext::new(Some(user), flash).noindex_nofollow();

    let encoding = upload::get_encoding(&conn);
    let queued = upload::get_queued_for_encoding(&conn, 50);
    let queued_count = upload::count_queued_for_encoding(&conn);
    let failed = upload::get_failed_encodings(&conn, 20);
    let legacy_count = upload::count_legacy_encoding(&conn);
    let machines = encoder_service::list_machines().ok();

    Ok(render!(admin::encoding(
        &ctx,
        encoding,
        queued,
        queued_count,
        failed,
        legacy_count,
        machines
    )))
}

#[rocket::post("/encoding/<file_id>/retry")]
pub(crate) fn action_retry_encoding(
    user: &User,
    conn: DatabaseConnection,
    file_id: String,
) -> Flash<Redirect> {
    if !user.is_admin() {
        return Flash::error(Redirect::to("/"), "");
    }

    match upload::requeue_failed_encoding(&conn, &file_id) {
        Ok(1) => {
            encoder_service::dispatch_in_background();
            Flash::success(Redirect::to("/admin/encoding"), "Queued for encoding.")
        }
        _ => Flash::error(Redirect::to("/admin/encoding"), "Could not queue upload."),
    }
}

#[rocket::post("/encoding/requeue_legacy")]
pub(crate) fn action_requeue_legacy_encodings(
    user: &User,
    conn: DatabaseConnection,
) -> Flash<Redirect> {
    if !user.is_admin() {
        return Flash::error(Redirect::to("/"), "");
    }

    match upload::requeue_legacy_encodings(&conn) {
        Ok(count) => {
            encoder_service::dispatch_in_background();
            Flash::success(
                Redirect::to("/admin/encoding"),
                format!("Queued {} uploads for encoding.", count),
            )
        }
        Err(_) => Flash::error(Redirect::to("/admin/encoding"), "Could not queue uploads."),
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
        Some(upload) => match encoder_service::start_encoder(&upload) {
            Ok(_) => Flash::success(Redirect::to("/"), "Sent video for encoding."),
            _ => Flash::error(Redirect::to("/"), "Could not enqueue video for encoding."),
        },
        None => return Flash::error(Redirect::to("/"), "Upload not found."),
    }
}

#[rocket::post("/actions/rebuild_md5")]
pub(crate) fn action_rebuild_md5(user: &User, conn: DatabaseConnection) -> Flash<Redirect> {
    if !user.is_admin() {
        return Flash::error(Redirect::to("/"), "");
    }

    match s3_client::list_objects() {
        Ok(upload_objects) => {
            for object in upload_objects.iter() {
                if let Err(e) = upload_service::update_md5(&conn, &object.file_id, &object.md5) {
                    warn!("[action_rebuild_md5] {}", e);
                }
            }
        }
        Err(e) => {
            warn!("[action_rebuild_md5] {}", e);
        }
    }

    Flash::success(
        Redirect::to("/"),
        "Rebuilding MD5 index. This will take some time.",
    )
}

pub(crate) fn router() -> Vec<rocket::Route> {
    rocket::routes![
        index,
        action_rebuild_tags,
        action_rebuild_tag_counts,
        action_encode_video,
        action_rebuild_md5,
        encoding,
        action_retry_encoding,
        action_requeue_legacy_encodings
    ]
}
