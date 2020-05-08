use std::env;
use std::io::{self, Write};

use warp::http::Response;
use warp::{path, Filter, Rejection, Reply};

use templates::RenderRucte;

#[derive(Debug)]
#[allow(dead_code)]
pub struct BuildInfo {
    build_timestamp: String,
    build_date: String,
    sha: String,
    sha_short: String,
    commit_date: String,
    target_triple: String,
    semver: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let default_host = "127.0.0.1".to_owned();
    let host = match env::var("HOST") {
        Ok(val) => val,
        Err(_) => default_host,
    };

    let default_port = 3030;
    let port = match env::var("PORT") {
        Ok(val) => match val.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                println!(
                    "the port number \"{}\" is invalid. default port will be used.",
                    val
                );
                default_port
            }
        },
        Err(_) => {
            println!("PORT is not defined in environment variables. default port will be used.",);
            default_port
        }
    };

    // GET /
    let index = warp::get().and(warp::path::end().and_then(index));

    // GET /public/:asset
    let assets = warp::get().and(warp::path("public").and(warp::fs::dir("./build/")));

    // All combined routes.
    let routes = index.or(assets);

    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    warp::serve(routes).run(addr).await;
}

async fn index() -> Result<impl Reply, Rejection> {
    Response::builder().html(|o| templates::index(o))
}

fn footer(out: &mut dyn Write) -> io::Result<()> {
    let build_info = BuildInfo {
        build_timestamp: env::var("VERGEN_BUILD_TIMESTAMP").unwrap(),
        build_date: env::var("VERGEN_BUILD_DATE").unwrap(),
        sha: env::var("VERGEN_SHA").unwrap(),
        sha_short: env::var("VERGEN_SHA_SHORT").unwrap(),
        commit_date: env::var("VERGEN_COMMIT_DATE").unwrap(),
        target_triple: env::var("VERGEN_TARGET_TRIPLE").unwrap(),
        semver: env::var("VERGEN_SEMVER").unwrap(),
    };

    templates::footer(out, build_info)
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
