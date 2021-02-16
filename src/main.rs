#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use std::env;

use lazy_static::lazy_static;
use rocket::http::{Cookie, Cookies, RawStr};
use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket::Rocket;
use rocket_contrib::serve::StaticFiles;

#[macro_use]
mod template_utils;

mod api;
mod config;
mod database;
mod ingestors;
mod models;
mod routes;
mod s3_client;
mod schema;
mod services;

use database::DatabaseConnection;
use models::upload_comment::RecentComment;
use models::user::{get_user_by_username, User};
use template_utils::{BaseContext, Pagination, Ructe};

embed_migrations!();

#[rocket::get("/?<page>&<q>")]
fn index(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    page: Option<&RawStr>,
    q: Option<String>,
) -> Ructe {
    lazy_static! {
        static ref UPLOADER_REGEX: regex::Regex =
            regex::Regex::new(r"(uploader:)([a-z_A-Z\d]*)\s?").unwrap();
    }

    let current_page = page.unwrap_or("1".into()).parse::<i64>().unwrap_or(1);
    let per_page = 50;
    let mut query = q.unwrap_or_default();

    // Check if the query has an `uploader:[USERNAME]` tag.
    let mut uploader: Option<User> = None;
    let mut recent_comments: Vec<RecentComment> = Vec::default();

    if !query.is_empty() {
        match UPLOADER_REGEX.captures(&query) {
            None => (),
            Some(matches) => {
                let full_match = &matches[0];
                let username = &matches[2];

                match get_user_by_username(&conn, &username) {
                    Some(user) => uploader = Some(user),
                    _ => (),
                }

                query = query.replace(full_match, "");
            }
        }
    } else {
        recent_comments = services::comment_service::get_recent_comments(&conn)
            .into_iter()
            .map(|i| i.into())
            .collect();
    }

    let (uploads, page_count, total_count) =
        models::upload::index(&conn, current_page, per_page, &query, uploader);

    let mut raw_tags: Vec<&str> = uploads
        .iter()
        .flat_map(|upload| upload.tag_string.split_whitespace().collect::<Vec<&str>>())
        .collect();

    raw_tags.sort();
    raw_tags.dedup();

    let tags = services::tag_service::by_names(&conn, &raw_tags);
    let (tag_groups, tags) = services::tag_service::group_tags(tags);

    let ctx = BaseContext::new(user, flash);
    let pagination = Pagination {
        current_page,
        page_count,
        total_count,
    };

    render!(page::index(
        &ctx,
        recent_comments,
        uploads,
        pagination,
        query,
        tags,
        tag_groups
    ))
}

#[rocket::post("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));

    Redirect::to("/")
}

#[rocket::get("/about")]
fn about(user: Option<&User>) -> Ructe {
    let ctx = BaseContext::new(user, None);

    render!(page::about(&ctx))
}

#[rocket::catch(404)]
fn not_found(req: &rocket::Request) -> Ructe {
    let user = req
        .guard::<Option<&User>>()
        .succeeded()
        .expect("Could not grab user.");

    let ctx = BaseContext::new(user, None);

    render!(error::not_found(&ctx))
}

fn run_db_migrations(rocket: rocket::Rocket) -> Result<Rocket, Rocket> {
    let conn = DatabaseConnection::get_one(&rocket).expect("No DB connection!");

    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            log::error!("Failed to run DB migrations: {:?}", e);
            Err(rocket)
        }
    }
}

#[rocket::get("/log?<page>")]
fn audit_log(conn: DatabaseConnection, user: Option<&User>, page: Option<&RawStr>) -> Ructe {
    let ctx = BaseContext::new(user, None);
    let current_page = page.unwrap_or("1".into()).parse::<i64>().unwrap_or(1);
    let per_page = 25;

    let log_count = services::audit_service::get_log_count(&conn);
    let logs = services::audit_service::get_paginated_log(&conn, current_page, per_page);
    let page_count = (log_count as f64 / per_page as f64).ceil() as i64;

    render!(page::log(&ctx, logs, page_count, current_page))
}

fn main() {
    dotenv::dotenv().ok();

    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();

    rocket::ignite()
        .attach(DatabaseConnection::fairing())
        .attach(rocket::fairing::AdHoc::on_attach(
            "DB Migrations",
            run_db_migrations,
        ))
        .mount(
            "/",
            rocket::routes![
                index,
                logout,
                about,
                audit_log,
                routes::login::index_redirect,
                routes::login::index,
                routes::login::post,
                routes::register::index_redirect,
                routes::register::index,
                routes::register::post,
                routes::upload::edit,
                routes::upload::embed,
                routes::upload::finalize,
                routes::upload::get,
                routes::upload::index,
                routes::upload::index_not_logged_in,
                routes::upload::log,
                routes::upload::update,
                routes::upload::upload,
                routes::upload::create_comment,
                routes::upload::edit_comment,
                routes::upload::edit_comment_page,
                routes::upload::delete,
                routes::upload::random,
                routes::webhooks::video::webhook,
            ],
        )
        .mount("/api/v1", routes::api::router())
        .mount("/queue", routes::queue::router())
        .mount("/admin", routes::admin::router())
        .mount("/user", routes::users::router())
        .mount("/tags", routes::tags::router())
        .mount(
            "/public",
            StaticFiles::from(format!("{}/{}", current_dir, "build")),
        )
        .register(rocket::catchers![not_found])
        .launch();
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
