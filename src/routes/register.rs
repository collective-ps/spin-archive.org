use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::invitation;
use crate::models::user::{self, RegistrationError, RegistrationFields};

#[rocket::get("/register")]
pub(crate) fn index_redirect(_user: &user::User) -> Redirect {
    Redirect::to("/")
}

#[rocket::get("/register?<code>", rank = 2)]
pub(crate) fn index(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    code: Option<String>,
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);

    if code.is_none() {
        return Err(Redirect::to("/"));
    }

    let code = code.unwrap();

    let invitation = invitation::by_code(&conn, &code);

    if invitation.is_some() {
        context.insert("code", &code);

        Ok(Template::render("register", &context))
    } else {
        Err(Redirect::to("/"))
    }
}

#[rocket::post("/register", data = "<form>")]
pub(crate) fn post(conn: DatabaseConnection, form: Form<RegistrationFields>) -> Flash<Redirect> {
    // @TODO(vy): Move username validation to somewhere else.
    let re = regex::Regex::new(r"^[a-z_A-Z\d]*$").unwrap();
    let register_url = format!("/register?code={}", form.code);
    let invitation = invitation::by_code(&conn, &form.code);

    if invitation.is_none() {
        return Flash::error(Redirect::to("/login"), "Invalid invitation code");
    }

    if !re.is_match(&form.username) || form.username.len() > 20 {
        return Flash::error(
        Redirect::to(register_url),
      "Invalid username. Must be no longer than 20 characters, and only contain letters + numbers + underscores.",
    );
    }

    match user::register(&conn, form.into_inner()) {
        Ok(user) => {
            let invitation = invitation.unwrap();

            let _ = invitation::update(
                &conn,
                invitation.id,
                &invitation::UpdateInvitation {
                    consumer_id: user.id,
                },
            );

            Flash::success(Redirect::to("/login"), "Account created!")
        }
        Err(err) => match err {
            RegistrationError::PasswordFailure => Flash::error(
                Redirect::to(register_url),
                "Invalid password or username. Please double-check.",
            ),
            RegistrationError::AlreadyExists => Flash::error(
                Redirect::to(register_url),
                "User already has taken that username/email.",
            ),
        },
    }
}
