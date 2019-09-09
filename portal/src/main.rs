#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use std::env;
#[cfg(debug_assertions)]
use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self};
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use env_logger::Env;
use lcutil::Result;

mod api;
mod camera;
mod schema;
mod templates;
mod ui;


embed_migrations!();


type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;


fn main() -> Result<()> {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    #[cfg(debug_assertions)]
    let static_dir = std::env::var("LC_STATIC")?;

    let templates = templates::load()?;

    // Create database connection pool
    let state_dir = env::var("STATE_DIRECTORY")?;
    let db_url = format!("{}/portal.db", state_dir);
    let pool = Pool::new(ConnectionManager::new(db_url))?;

    // Ensure database is initialized
    {
        let conn = pool.get()?;
        embedded_migrations::run(&conn)?;
    }

    HttpServer::new(move || {
            let app = App::new();
            let app = app.service(web::scope("/api").configure(api::configure(pool.clone())));
            #[cfg(debug_assertions)]
            let app = app.service(Files::new("/static", &static_dir));
            let app = app.configure(ui::configure(templates.clone(), pool.clone()));

            app
        })
        .bind("127.0.0.1:9351")?
        .run()?;

    Ok(())
}
