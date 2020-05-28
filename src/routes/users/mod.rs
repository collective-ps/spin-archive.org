use rocket::http::RawStr;
use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::user::{get_user_by_username, User};
use crate::services::{comment_service, upload_service};

#[rocket::get("/<username>")]
pub(crate) fn index(
  conn: DatabaseConnection,
  flash: Option<FlashMessage>,
  user: Option<&User>,
  username: String,
) -> Result<Template, Redirect> {
  let mut context = TeraContext::new();

  context::flash_context(&mut context, flash);
  context::user_context(&mut context, user);

  match get_user_by_username(&conn, &username) {
    Some(profile_user) => {
      let comment_count = comment_service::get_comment_count_by_user_id(&conn, profile_user.id);
      let upload_count = upload_service::get_upload_count_by_user_id(&conn, profile_user.id);

      context.insert("profile_user", &profile_user);
      context.insert("comment_count", &comment_count);
      context.insert("upload_count", &upload_count);

      Ok(Template::render("users/profile", &context))
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
) -> Result<Template, Redirect> {
  let mut context = TeraContext::new();
  let current_page = page.unwrap_or("1".into()).parse::<i64>().unwrap_or(1);
  let per_page = 25;

  context::flash_context(&mut context, flash);
  context::user_context(&mut context, user);

  match get_user_by_username(&conn, &username) {
    Some(profile_user) => {
      let comment_count = comment_service::get_comment_count_by_user_id(&conn, profile_user.id);
      let upload_count = upload_service::get_upload_count_by_user_id(&conn, profile_user.id);
      let comments_with_uploads =
        comment_service::get_paginated_comments(&conn, profile_user.id, current_page, per_page);

      let page_count = (comment_count as f64 / per_page as f64).ceil() as i64;

      context.insert("profile_user", &profile_user);
      context.insert("comment_count", &comment_count);
      context.insert("upload_count", &upload_count);
      context.insert("comments_with_uploads", &comments_with_uploads);
      context.insert("page_count", &page_count);
      context.insert("page", &current_page);

      Ok(Template::render("users/comments", &context))
    }
    _ => Err(Redirect::to("/404")),
  }
}

pub(crate) fn router() -> Vec<rocket::Route> {
  rocket::routes![index, comments]
}
