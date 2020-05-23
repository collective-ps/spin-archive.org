use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::{User, UserRole};
use crate::services::tag_service;

/// Admin area.
#[rocket::get("/")]
pub(crate) fn index(flash: Option<FlashMessage>, user: &User) -> Result<Template, Redirect> {
  if user.role == UserRole::Admin {
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
  if user.role == UserRole::Admin {
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
  if user.role == UserRole::Admin {
    tag_service::rebuild_tag_counts(&conn);

    Flash::success(Redirect::to("/admin"), "Rebuilt tag counts!")
  } else {
    Flash::error(Redirect::to("/"), "")
  }
}

pub(crate) fn router() -> Vec<rocket::Route> {
  rocket::routes![index, action_rebuild_tags, action_rebuild_tag_counts]
}
