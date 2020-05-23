use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseConnection;
use crate::models::tag;

#[derive(Serialize, Deserialize)]
pub struct SuggestionResponse {
  tags: Vec<String>,
}

#[rocket::get("/tags/suggestions?<q>")]
pub fn suggestions(
  conn: DatabaseConnection,
  q: Option<String>,
) -> Result<Json<SuggestionResponse>, BadRequest<()>> {
  let prefix = q.unwrap_or_default();
  let tags = tag::starting_with(&conn, &prefix)
    .iter()
    .map(|tag| tag.name.clone())
    .collect();

  let response = SuggestionResponse { tags };

  Ok(Json(response))
}
