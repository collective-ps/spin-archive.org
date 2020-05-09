use std::env;

pub fn secret_key() -> String {
  env::var("SECRET_KEY").unwrap_or("SECRET_KEY_PLEASE_CHANGE_ME".to_owned())
}
