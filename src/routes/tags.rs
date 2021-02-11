use rocket::request::FlashMessage;

use crate::database::DatabaseConnection;
use crate::models::user::User;
use crate::services::tag_service;
use crate::template_utils::{BaseContext, Ructe};

#[rocket::get("/")]
pub(crate) fn index(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
) -> Ructe {
    let ctx = BaseContext::new(user, flash);
    let tags = tag_service::all(&conn);
    let (tag_groups, tags) = tag_service::group_tags(tags);

    render!(tags::index(&ctx, tag_groups, tags))
}

pub(crate) fn router() -> Vec<rocket::Route> {
    rocket::routes![index]
}
