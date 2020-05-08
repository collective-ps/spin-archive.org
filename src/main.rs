#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::HashMap;
use std::env;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::tera::{
    Context as TeraContext, Result as TeraResult, Value as TeraValue,
};
use rocket_contrib::templates::Template;
use serde::Serialize;

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
}

#[rocket::get("/")]
fn index() -> Template {
    let context = TeraContext::new();
    Template::render("index", &context)
}

fn main() {
    rocket::ignite()
        .mount("/", rocket::routes![index])
        .mount(
            "/public",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/build")),
        )
        .attach(Template::custom(|engines| {
            engines.tera.register_function("footer_data", footer_data());
        }))
        .launch();
}

fn footer_data() -> GlobalFn {
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
