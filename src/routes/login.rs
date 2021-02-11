use rocket::http::{Cookie, Cookies};
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};

use crate::database::DatabaseConnection;
use crate::models::user::{self, LoginError};
use crate::template_utils::{BaseContext, Ructe};

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
pub(crate) fn index(flash: Option<FlashMessage>) -> Ructe {
    let ctx = BaseContext::new(None, flash);

    render!(page::login(&ctx))
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
