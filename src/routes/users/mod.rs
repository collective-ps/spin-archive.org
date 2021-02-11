use rocket::http::RawStr;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};

use crate::database::DatabaseConnection;
use crate::models::user::{get_user_by_username, User};
use crate::services::{api_token_service, comment_service, upload_service};
use crate::template_utils::{BaseContext, Ructe};

#[rocket::get("/settings")]
pub(crate) fn settings(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: &User,
) -> Result<Ructe, Redirect> {
    let ctx = BaseContext::new(Some(user), flash);
    let api_tokens = api_token_service::get_tokens_by_user(&conn, user.id);

    Ok(render!(users::settings(&ctx, api_tokens)))
}

#[rocket::post("/settings/api_tokens")]
pub(crate) fn new_api_token(conn: DatabaseConnection, user: &User) -> Flash<Redirect> {
    if user.is_contributor() {
        match api_token_service::new(&conn, &user) {
            Ok(_) => Flash::success(Redirect::to("/user/settings"), "Generated an API token"),
            Err(_) => Flash::error(
                Redirect::to("/user/settings"),
                "Could not generate an API token.",
            ),
        }
    } else {
        Flash::error(
      Redirect::to("/user/settings"),
      "You do not have permission to create an API token. Must be at least [Contributor] rank.",
    )
    }
}

#[rocket::post("/settings/api_tokens/<id>/delete")]
pub(crate) fn delete_api_token(conn: DatabaseConnection, id: i64, user: &User) -> Flash<Redirect> {
    if user.is_contributor() {
        match api_token_service::revoke(&conn, user.id, id) {
            Ok(_) => Flash::success(
                Redirect::to("/user/settings"),
                "Deleted API token succesfully.",
            ),
            Err(_) => Flash::error(
                Redirect::to("/user/settings"),
                "Could not delete API token.",
            ),
        }
    } else {
        Flash::error(
      Redirect::to("/user/settings"),
      "You do not have permission to delete an API token. Must be at least [Contributor] rank.",
    )
    }
}

#[rocket::get("/<username>")]
pub(crate) fn index(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    username: String,
) -> Result<Ructe, Redirect> {
    let ctx = BaseContext::new(user, flash);

    match get_user_by_username(&conn, &username) {
        Some(profile_user) => {
            let comment_count =
                comment_service::get_comment_count_by_user_id(&conn, profile_user.id);
            let upload_count = upload_service::get_upload_count_by_user_id(&conn, profile_user.id);

            Ok(render!(users::profile(
                &ctx,
                profile_user,
                comment_count,
                upload_count
            )))
        }
        _ => Err(Redirect::to("/404")),
    }
}

#[rocket::get("/<username>/comments?<page>")]
pub(crate) fn comments(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    username: String,
    page: Option<&RawStr>,
) -> Result<Ructe, Redirect> {
    let ctx = BaseContext::new(user, flash);
    let current_page = page.unwrap_or("1".into()).parse::<i64>().unwrap_or(1);
    let per_page = 25;

    match get_user_by_username(&conn, &username) {
        Some(profile_user) => {
            let comments_with_uploads = comment_service::get_paginated_comments(
                &conn,
                profile_user.id,
                current_page,
                per_page,
            );

            let comment_count =
                comment_service::get_comment_count_by_user_id(&conn, profile_user.id);
            let page_count = (comment_count as f64 / per_page as f64).ceil() as i64;

            Ok(render!(users::comments(
                &ctx,
                comments_with_uploads,
                profile_user,
                page_count,
                current_page
            )))
        }
        _ => Err(Redirect::to("/404")),
    }
}

pub(crate) fn router() -> Vec<rocket::Route> {
    rocket::routes![index, comments, settings, new_api_token, delete_api_token,]
}
