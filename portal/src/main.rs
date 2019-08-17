#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate diesel;

use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self};
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

fn main()
{
    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    let mut hotwatch = Hotwatch::new()
        .expect("main: failed to initialize Hotwatch");
    let templates = Templates::load(&mut hotwatch);
    let static_dir = std::env::var("LC_STATIC")
        .expect("main: could not read LC_STATIC");

    HttpServer::new(move || {
            App::new()
                .service(web::scope("/api").configure(api::configure))
                .service(Files::new("/static", &static_dir))
                .configure(ui::configure(templates.clone()))
        })
        .bind("127.0.0.1:8000").unwrap()
        .run().unwrap()
}
