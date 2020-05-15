use std::collections::HashMap;

use rocket::request::FlashMessage;

use crate::models::user::User;

pub(crate) fn flash_context(context: &mut HashMap<String, String>, flash: Option<FlashMessage>) {
  if let Some(msg) = flash {
    context.insert("flash_name".into(), msg.name().into());
    context.insert("flash_message".into(), msg.msg().into());
  }
}

pub(crate) fn user_context(context: &mut HashMap<String, String>, user: Option<&User>) {
  if let Some(user) = user {
    context.insert("user_id".into(), user.id.to_string().into());
    context.insert("user_role".into(), user.role.to_string().into());
    context.insert("user_can_upload".into(), user.can_upload().to_string());
    context.insert("username".into(), user.username.clone().into());
  }
}
