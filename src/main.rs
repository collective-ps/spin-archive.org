#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel_migrations;

use std::collections::HashMap;
use std::env;

use rocket::Rocket;
use rocket_contrib::databases::diesel;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::tera::{
    Context as TeraContext, Result as TeraResult, Value as TeraValue,
};
use rocket_contrib::templates::Template;
use serde::Serialize;

embed_migrations!();

type GlobalFn = Box<dyn Fn(HashMap<String, TeraValue>) -> TeraResult<TeraValue> + Sync + Send>;

#[rocket_contrib::database("spin_archive")]
struct DatabaseConnection(diesel::PgConnection);

#[derive(Debug, Serialize)]
pub struct BuildInfo {
    build_timestamp: String,
    build_date: String,
    sha: String,
    sha_short: String,
    commit_date: String,
    target_triple: String,
    semver: String,
}

#[rocket::get("/")]
fn index() -> Template {
    let context = TeraContext::new();
    Template::render("index", &context)
}

#[rocket::catch(404)]
fn not_found(_req: &rocket::Request) -> Template {
    let context = TeraContext::new();
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
    rocket::ignite()
        .attach(DatabaseConnection::fairing())
        .attach(Template::custom(|engines| {
            engines.tera.register_function("build_info", build_info());
        }))
        .attach(rocket::fairing::AdHoc::on_attach(
            "DB Migrations",
            run_db_migrations,
        ))
        .mount("/", rocket::routes![index])
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
        let build_info = BuildInfo {
            build_timestamp: env::var("VERGEN_BUILD_TIMESTAMP").unwrap(),
            build_date: env::var("VERGEN_BUILD_DATE").unwrap(),
            sha: env::var("VERGEN_SHA").unwrap(),
            sha_short: env::var("VERGEN_SHA_SHORT").unwrap(),
            commit_date: env::var("VERGEN_COMMIT_DATE").unwrap(),
            target_triple: env::var("VERGEN_TARGET_TRIPLE").unwrap(),
            semver: env::var("VERGEN_SEMVER").unwrap(),
        };

        Ok(serde_json::to_value(build_info).unwrap())
    })
}
