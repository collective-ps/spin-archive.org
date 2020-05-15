use rocket::http::{Cookie, Cookies};
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::{self, LoginError};

#[derive(rocket::FromForm)]
pub(crate) struct LoginFields {
  username: String,
  password: String,
}

#[rocket::get("/login")]
pub(crate) fn index_redirect(_user: &user::User) -> Redirect {
  Redirect::to("/")
}

#[rocket::get("/login", rank = 2)]
pub(crate) fn index(flash: Option<FlashMessage>) -> Template {
  let mut context = TeraContext::new();
  context::flash_context(&mut context, flash);

  Template::render("login", &context)
}

#[rocket::post("/login", data = "<form>")]
pub(crate) fn post(
  mut cookies: Cookies,
  conn: DatabaseConnection,
  form: Form<LoginFields>,
) -> Flash<Redirect> {
  match user::login(&conn, &form.username, &form.password) {
    Ok(user) => {
      cookies.add_private(Cookie::new("user_id", user.id.to_string()));
      Flash::success(Redirect::to("/"), "")
    }
    Err(err) => match err {
      LoginError::InvalidPasswordOrUser => {
        Flash::error(Redirect::to("/login"), "Invalid password or username.")
      }
    },
  }
}
