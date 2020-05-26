use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::upload::UploadStatus;
use crate::models::user::User;
use crate::services::upload_service;

#[rocket::get("/")]
pub(crate) fn index(
  conn: DatabaseConnection,
  flash: Option<FlashMessage>,
  user: &User,
) -> Result<Template, Redirect> {
  if !user.is_contributor() {
    return Err(Redirect::to("/"));
  }

  let mut context = TeraContext::new();
  context::flash_context(&mut context, flash);
  context::user_context(&mut context, Some(user));

  let uploads_with_users = upload_service::get_pending_approval_uploads(&conn);
  context.insert("uploads_with_users", &uploads_with_users);

  Ok(Template::render("queue/index", &context))
}

#[rocket::post("/<file_id>/approve")]
pub(crate) fn approve(conn: DatabaseConnection, user: &User, file_id: String) -> Flash<Redirect> {
  if !user.is_contributor() {
    return Flash::error(Redirect::to("/"), "");
  }

  match upload_service::get_by_file_id(&conn, &file_id)
    .ok_or("Upload not found.")
    .and_then(|upload| {
      if upload.status == UploadStatus::PendingApproval {
        upload_service::update_status(&conn, upload.id, UploadStatus::Completed)
          .map_err(|_| "Could not change upload status to Completed.")
      } else {
        Err("Already approved.")
      }
    })
    .map(|_| "Approved!")
  {
    Ok(message) => Flash::success(Redirect::to("/queue"), message),
    Err(message) => Flash::error(Redirect::to("/queue"), message),
  }
}

pub(crate) fn router() -> Vec<rocket::Route> {
  rocket::routes![index, approve]
}