#[macro_use]
extern crate diesel;

#[cfg(debug_assertions)]
use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self};
use env_logger::Env;
use lunacam::Result;
use lunacam::db;

mod api;
mod camera;
mod templates;
mod ui;


fn main() -> Result<()> {

    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    #[cfg(debug_assertions)]
    let static_dir = std::env::var("LC_STATIC")?;

    let templates = templates::load()?;
    let pool = db::connect()?;

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
