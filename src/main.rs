#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use std::collections::HashMap;
use std::env;

use lazy_static::lazy_static;
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

lazy_static! {
    #[derive(Copy, Clone, Debug)]
    static ref APP_VERSION: String = env::var("GIT_REV").unwrap_or_default();
}

#[rocket::get("/?<page>&<q>")]
fn index(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    page: Option<&RawStr>,
    q: Option<String>,
) -> Template {
    let mut context = TeraContext::new();
    let current_page = page.unwrap_or("1".into()).parse::<i64>().unwrap_or(1);
    let per_page = 25;
    let (uploads, users, page_count) =
        models::upload::index(&conn, current_page, per_page, q.clone());
    let query = q.unwrap_or_default();

    let mut raw_tags: Vec<&str> = uploads
        .iter()
        .flat_map(|upload| upload.tag_string.split_whitespace().collect::<Vec<&str>>())
        .collect();

    raw_tags.sort();
    raw_tags.dedup();

    let tags = services::tag_service::by_names(&conn, &raw_tags);
    let (tag_groups, tags) = services::tag_service::group_tags(tags);

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    context.insert("uploads", &uploads);
    context.insert("page_count", &page_count);
    context.insert("total_count", &(page_count * per_page));
    context.insert("page", &current_page);
    context.insert("tags", &tags);
    context.insert("tag_groups", &tag_groups);
    context.insert("query", &query);
    context.insert("users", &users);

    Template::render("index", &context)
}

#[rocket::post("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));
    Redirect::to("/")
}

#[rocket::get("/about")]
fn about(user: Option<&User>) -> Template {
    let mut context = TeraContext::new();
    context::user_context(&mut context, user);

    Template::render("about", &context)
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

    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();

    rocket::ignite()
        .attach(DatabaseConnection::fairing())
        .attach(Template::custom(|engines| {
            engines
                .tera
                .register_function("get_thumbnail_url", context::get_thumbnail_url());
            engines
                .tera
                .register_function("get_file_url", context::get_file_url());
            engines
                .tera
                .register_function("get_video_url", context::get_video_url());
            engines
                .tera
                .register_function("is_video", context::is_video());
            engines
                .tera
                .register_function("split_tags", context::split_tags());

            engines.tera.register_filter("tag_url", context::tag_url);
            engines
                .tera
                .register_filter("humanized_past", context::humanized_past);

            engines
                .tera
                .register_filter("from_markdown", context::from_markdown);

            engines
                .tera
                .register_filter("append_version", append_version);
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
                about,
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
                routes::webhooks::video::webhook,
            ],
        )
        .mount("/api/v1", routes::api::router())
        .mount("/admin", routes::admin::router())
        .mount(
            "/public",
            StaticFiles::from(format!("{}/{}", current_dir, "build")),
        )
        .register(rocket::catchers![not_found])
        .launch();
}

pub fn append_version(
    value: TeraValue,
    _args: HashMap<String, TeraValue>,
) -> TeraResult<TeraValue> {
    match serde_json::from_value::<String>(value.clone()) {
        Ok(content) => {
            let new_string = format!("{}?v={}", content, &**APP_VERSION);
            Ok(serde_json::to_value(new_string).unwrap())
        }
        Err(_) => Err("Could not get string.".into()),
    }
}
