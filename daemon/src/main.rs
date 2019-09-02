#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

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


embed_migrations!();


fn main() {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    let state_dir = std::env::var("STATE_DIRECTORY")
        .unwrap();
    let db_url = format!("{}/daemon.db", state_dir);
    let db_conn = SqliteConnection::establish(&db_url)
        .unwrap();
    embedded_migrations::run(&db_conn)
        .unwrap();

    let rt = Runtime::new()
        .unwrap();

    transcoder::initialize(Box::new(rt.executor()), db_conn)
        .unwrap();

    HttpServer::new(||
            App::new()
                .service(web::scope("/api").configure(api::configure))
        )
        .bind("127.0.0.1:9350").unwrap()
        .run().unwrap();
}
