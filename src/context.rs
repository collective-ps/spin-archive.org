use std::collections::HashMap;

use rocket::request::FlashMessage;

pub(crate) fn flash_context(context: &mut HashMap<String, String>, flash: Option<FlashMessage>) {
  if let Some(msg) = flash {
    context.insert("flash_name".into(), msg.name().into());
    context.insert("flash_message".into(), msg.msg().into());
  }
}
