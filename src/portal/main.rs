#[macro_use]
extern crate diesel;

use std::sync::Arc;
#[cfg(debug_assertions)]
use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self};
use lunacam::Result;
use lunacam::db;
use lunacam::logging;
use tera::Tera;

mod api;
mod camera;
mod ui;


fn main() -> Result<()> {

    logging::init();

    #[cfg(debug_assertions)]
    let static_dir = std::env::var("LC_STATIC")?;

    let template_dir = std::env::var("LC_TEMPLATES")?;
    let templates = Arc::new(Tera::new(&format!("{}/**/*", template_dir))?);

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
