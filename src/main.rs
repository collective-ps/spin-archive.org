#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use std::collections::HashMap;
use std::env;

use rocket::http::RawStr;
use rocket::http::{Cookie, Cookies};
use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket::Rocket;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::tera::{
    Context as TeraContext, Result as TeraResult, Value as TeraValue,
};
use rocket_contrib::templates::Template;
use serde::Serialize;

mod config;
mod context;
mod database;
mod models;
mod pagination;
mod routes;
mod s3_client;
mod schema;
mod services;

use database::DatabaseConnection;
use models::user::User;

embed_migrations!();

type GlobalFn = Box<dyn Fn(HashMap<String, TeraValue>) -> TeraResult<TeraValue> + Sync + Send>;

#[derive(Debug, Serialize)]
pub struct BuildInfo {
    build_timestamp: String,
    build_date: String,
    sha: String,
    sha_short: String,
    commit_date: String,
    target_triple: String,
    semver: String,
    commit_url: String,
}

#[rocket::get("/?<page>")]
fn index(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    page: Option<&RawStr>,
) -> Template {
    let mut context = TeraContext::new();
    let current_page = page.unwrap_or("1".into()).parse::<i64>().unwrap_or(1);
    let (uploads, page_count) = models::upload::index(&conn, current_page).unwrap();

    let mut tags: Vec<&str> = uploads
        .iter()
        .flat_map(|upload| upload.tag_string.split_whitespace().collect::<Vec<&str>>())
        .collect();

    tags.sort();
    tags.dedup();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    context.insert("uploads", &uploads);
    context.insert("page_count", &page_count);
    context.insert("page", &current_page);
    context.insert("tags", &tags);

    Template::render("index", &context)
}

#[rocket::post("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));
    Redirect::to("/")
}

#[rocket::catch(404)]
fn not_found(req: &rocket::Request) -> Template {
    let mut context = TeraContext::new();
    let user = req.guard::<Option<&User>>().succeeded();

    if let Some(user) = user {
        context::user_context(&mut context, user);
    }

    Template::render("error/404", &context)
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

fn main() {
    dotenv::dotenv().ok();

    rocket::ignite()
        .attach(DatabaseConnection::fairing())
        .attach(Template::custom(|engines| {
            engines.tera.register_function("build_info", build_info());
            engines
                .tera
                .register_function("get_thumbnail_url", context::get_thumbnail_url());
            engines
                .tera
                .register_function("get_file_url", context::get_file_url());
            engines
                .tera
                .register_function("is_video", context::is_video());
            engines
                .tera
                .register_function("split_tags", context::split_tags());
        }))
        .attach(rocket::fairing::AdHoc::on_attach(
            "DB Migrations",
            run_db_migrations,
        ))
        .mount(
            "/",
            rocket::routes![
                index,
                logout,
                routes::login::index_redirect,
                routes::login::index,
                routes::login::post,
                routes::register::index_redirect,
                routes::register::index,
                routes::register::post,
                routes::upload::get,
                routes::upload::index,
                routes::upload::index_not_logged_in,
                routes::upload::upload,
                routes::upload::finalize,
            ],
        )
        .mount(
            "/public",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/build")),
        )
        .register(rocket::catchers![not_found])
        .launch();
}

/// Template function that returns information about the current release build.
fn build_info() -> GlobalFn {
    Box::new(move |_args| -> TeraResult<TeraValue> {
        let sha = env::var("VERGEN_SHA").unwrap();

        let build_info = BuildInfo {
            build_timestamp: env::var("VERGEN_BUILD_TIMESTAMP").unwrap(),
            build_date: env::var("VERGEN_BUILD_DATE").unwrap(),
            sha_short: env::var("VERGEN_SHA_SHORT").unwrap(),
            commit_date: env::var("VERGEN_COMMIT_DATE").unwrap(),
            target_triple: env::var("VERGEN_TARGET_TRIPLE").unwrap(),
            semver: env::var("VERGEN_SEMVER").unwrap(),
            commit_url: format!(
                "https://github.com/andrewvy/spin-archive.org/commit/{}",
                &sha
            ),
            sha,
        };

        Ok(serde_json::to_value(build_info).unwrap())
    })
}
