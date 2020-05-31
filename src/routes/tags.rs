use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::User;
use crate::services::tag_service;

#[rocket::get("/")]
pub(crate) fn index(
  conn: DatabaseConnection,
  flash: Option<FlashMessage>,
  user: Option<&User>,
) -> Result<Template, Redirect> {
  let mut context = TeraContext::new();
  context::flash_context(&mut context, flash);
  context::user_context(&mut context, user);

  let tags = tag_service::all(&conn);
  let (tag_groups, tags) = tag_service::group_tags(tags);

  context.insert("tags", &tags);
  context.insert("tag_groups", &tag_groups);

  Ok(Template::render("tags/index", &context))
}

pub(crate) fn router() -> Vec<rocket::Route> {
  rocket::routes![index]
}
