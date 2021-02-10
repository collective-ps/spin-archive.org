use std::convert::TryInto;

use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::{self, RegistrationError, RegistrationFields};

#[rocket::get("/register")]
pub(crate) fn index_redirect(_user: &user::User) -> Redirect {
    Redirect::to("/")
}

#[rocket::get("/register", rank = 2)]
pub(crate) fn index(flash: Option<FlashMessage>) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();
    context::flash_context(&mut context, flash);

    Ok(Template::render("register", &context))
}

#[rocket::post("/register", data = "<form>")]
pub(crate) fn post(conn: DatabaseConnection, form: Form<RegistrationFields>) -> Flash<Redirect> {
    // @TODO(vy): Move username validation to somewhere else.
    let re = regex::Regex::new(r"^[a-z_A-Z\d]*$").unwrap();
    const REGISTER_URL: &'static str = "/register";

    if !re.is_match(&form.username) || form.username.len() > 20 {
        return Flash::error(
        Redirect::to(REGISTER_URL),
      "Invalid username. Must be no longer than 20 characters, and only contain letters + numbers + underscores.",
    );
    }

    match form
        .into_inner()
        .try_into()
        .and_then(|params: user::NewUser| user::register(&conn, params))
    {
        Ok(_user) => Flash::success(Redirect::to("/login"), "Account created!"),
        Err(err) => match err {
            RegistrationError::PasswordFailure => Flash::error(
                Redirect::to(REGISTER_URL),
                "Invalid password or username. Please double-check.",
            ),
            RegistrationError::AlreadyExists => Flash::error(
                Redirect::to(REGISTER_URL),
                "User already has taken that username/email.",
            ),
        },
    }
}
