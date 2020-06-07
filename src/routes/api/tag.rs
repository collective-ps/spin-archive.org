use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseConnection;
use crate::models::tag;

#[derive(Serialize, Deserialize)]
pub struct TagJson {
    name: String,
    upload_count: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SuggestionResponse {
    tags: Vec<TagJson>,
}

#[rocket::get("/tags/suggestions?<q>")]
pub fn suggestions(
    conn: DatabaseConnection,
    q: Option<String>,
) -> Result<Json<SuggestionResponse>, BadRequest<()>> {
    let prefix = q.unwrap_or_default();
    let tags = tag::starting_with(&conn, &prefix, 10)
        .iter()
        .map(|tag| TagJson {
            name: tag.name.clone(),
            upload_count: tag.upload_count,
        })
        .collect();

    let response = SuggestionResponse { tags };

    Ok(Json(response))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![suggestions]
}
