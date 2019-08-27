#[macro_use]
extern crate diesel;

use actix_web::{App, HttpServer};
use actix_web::web;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use env_logger::Env;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Runtime};

#[macro_use]
mod macros;

mod api;
mod schema;
mod settings;
mod transcoder;


// TODO: move to lc_common crate
#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Orientation {
    Landscape,
    Portrait,
    InvertedLandscape,
    InvertedPortrait,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Landscape
    }
}


fn main() {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    let db_url = std::env::var("DATABASE_URL")
        .unwrap();
    let db_conn = SqliteConnection::establish(&db_url)
        .unwrap();

    let rt = Runtime::new()
        .unwrap();

    transcoder::initialize(Box::new(rt.executor()), db_conn)
        .unwrap();

    HttpServer::new(||
            App::new()
                .service(web::scope("/api").configure(api::configure))
        )
        .bind("127.0.0.1:8000").unwrap()
        .run().unwrap();
}
