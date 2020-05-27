use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::{get_user_by_username, User};
use crate::services::{comment_service, upload_service};

#[rocket::get("/<username>")]
pub(crate) fn index(
  conn: DatabaseConnection,
  flash: Option<FlashMessage>,
  user: Option<&User>,
  username: String,
) -> Result<Template, Redirect> {
  let mut context = TeraContext::new();

  context::flash_context(&mut context, flash);
  context::user_context(&mut context, user);

  match get_user_by_username(&conn, &username) {
    Some(profile_user) => {
      let comment_count = comment_service::get_comment_count_by_user_id(&conn, profile_user.id);
      let upload_count = upload_service::get_upload_count_by_user_id(&conn, profile_user.id);

      context.insert("profile_user", &profile_user);
      context.insert("comment_count", &comment_count);
      context.insert("upload_count", &upload_count);

      Ok(Template::render("users/profile", &context))
    }
    _ => Err(Redirect::to("/404")),
  }
}

pub(crate) fn router() -> Vec<rocket::Route> {
  rocket::routes![index]
}
