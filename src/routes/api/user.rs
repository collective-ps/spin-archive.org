use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::api::Auth;

#[derive(Serialize, Deserialize)]
pub struct UserJson {
  id: i32,
  username: String,
  role: String,
}

#[rocket::get("/me")]
pub fn me(auth: Auth) -> Result<Json<UserJson>, BadRequest<()>> {
  let user = UserJson {
    id: auth.user.id,
    username: auth.user.username.clone(),
    role: auth.user.role.to_string(),
  };

  Ok(Json(user))
}

pub fn routes() -> Vec<rocket::Route> {
  rocket::routes![me]
}
