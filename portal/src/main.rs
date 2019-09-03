#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

#[cfg(debug_assertions)]
use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use env_logger::Env;
use hotwatch::{Hotwatch};

#[macro_use]
mod macros;

mod api;
mod camera;
mod schema;
mod templates;
mod ui;

use crate::templates::Templates;


embed_migrations!();


fn main() {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    let mut hotwatch = Hotwatch::new()
        .expect("main: failed to initialize Hotwatch");
    let templates = Templates::load(&mut hotwatch);

    #[cfg(debug_assertions)]
    let static_dir = std::env::var("LC_STATIC")
        .expect("main: could not read LC_STATIC");

    let state_dir = std::env::var("STATE_DIRECTORY")
        .unwrap();
    let db_url = format!("{}/portal.db", state_dir);
    let db_conn = SqliteConnection::establish(&db_url)
        .unwrap();
    embedded_migrations::run(&db_conn)
        .unwrap();

    HttpServer::new(move || {
            let app = App::new();
            let app = app.service(web::scope("/api").configure(api::configure(db_url.clone())));
            #[cfg(debug_assertions)]
            let app = app.service(Files::new("/static", &static_dir));
            let app = app.configure(ui::configure(templates.clone(), db_url.clone()));

            app
        })
        .bind("127.0.0.1:9351").unwrap()
        .run().unwrap()
}
