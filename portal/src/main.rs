#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use std::env;
#[cfg(debug_assertions)]
use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self};
use diesel::r2d2::{ConnectionManager, Pool};
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


type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;


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

    // Create database connection pool
    let state_dir = env::var("STATE_DIRECTORY")
        .expect("failed to read STATE_DIRECTORY environment variable");
    let db_url = format!("{}/portal.db", state_dir);
    let pool = Pool::builder()
        .build(ConnectionManager::new(db_url))
        .expect("failed to create database connection pool");

    // Ensure database is initialized
    {
        let conn = pool.get()
            .expect("failed to open initial database connection");
        embedded_migrations::run(&conn)
            .expect("failed to initialize database");
    }

    HttpServer::new(move || {
            let app = App::new();
            let app = app.service(web::scope("/api").configure(api::configure(pool.clone())));
            #[cfg(debug_assertions)]
            let app = app.service(Files::new("/static", &static_dir));
            let app = app.configure(ui::configure(templates.clone(), pool.clone()));

            app
        })
        .bind("127.0.0.1:9351").unwrap()
        .run().unwrap()
}
