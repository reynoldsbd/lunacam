#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use actix_web::{App, HttpServer};
use actix_web::web;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use env_logger::Env;
use lcutil::Result;
use tokio::runtime::{Runtime};

mod api;
mod schema;
mod settings;
mod transcoder;


embed_migrations!();


fn main() -> Result<()> {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    let state_dir = std::env::var("STATE_DIRECTORY")?;
    let db_url = format!("{}/daemon.db", state_dir);
    let db_conn = SqliteConnection::establish(&db_url)?;
    embedded_migrations::run(&db_conn)?;

    let rt = Runtime::new()?;

    transcoder::initialize(Box::new(rt.executor()), db_conn)?;

    HttpServer::new(||
            App::new()
                .service(web::scope("/api").configure(api::configure))
        )
        .bind("127.0.0.1:9350")?
        .run()?;

    Ok(())
}
