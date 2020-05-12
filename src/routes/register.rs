use std::collections::HashMap;

use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::{self, RegistrationError, RegistrationFields};

#[rocket::get("/register")]
pub(crate) fn index_redirect(_user: &user::User) -> Redirect {
  Redirect::to("/")
}

#[rocket::get("/register", rank = 2)]
pub(crate) fn index(flash: Option<FlashMessage>) -> Template {
  let mut context: HashMap<String, String> = HashMap::new();
  context::flash_context(&mut context, flash);

  Template::render("register", &context)
}

#[rocket::post("/register", data = "<form>")]
pub(crate) fn post(conn: DatabaseConnection, form: Form<RegistrationFields>) -> Flash<Redirect> {
  match user::register(&conn, form.into_inner()) {
    Ok(_) => Flash::success(Redirect::to("/"), ""),
    Err(err) => match err {
      RegistrationError::PasswordFailure => Flash::error(
        Redirect::to("/register"),
        "Invalid password or username. Please double-check.",
      ),
      RegistrationError::AlreadyExists => Flash::error(
        Redirect::to("/register"),
        "User already has taken that username/email.",
      ),
    },
  }
}
