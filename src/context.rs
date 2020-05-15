use rocket::request::FlashMessage;
use rocket_contrib::templates::tera::Context as TeraContext;

use crate::models::user::User;

pub(crate) fn flash_context(context: &mut TeraContext, flash: Option<FlashMessage>) {
  if let Some(msg) = flash {
    context.insert("flash_name", msg.name());
    context.insert("flash_message", msg.msg());
  }
}

pub(crate) fn user_context(context: &mut TeraContext, user: Option<&User>) {
  if let Some(user) = user {
    context.insert("user_id", &user.id.to_string());
    context.insert("user_role", &user.role.to_string());
    context.insert("user_can_upload", &user.can_upload());
    context.insert("username", &user.username.clone());
  }
}
