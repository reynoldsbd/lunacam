#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate diesel;

use actix_files::{Files};
use actix_web::{App, HttpServer, Responder};
use actix_web::web::{self, Data};
use env_logger::Env;
use hotwatch::{Hotwatch};
use serde::{Serialize};
use tera::{Context};

#[macro_use]
mod macros;

mod api;
mod camera;
mod schema;
mod templates;

use crate::templates::Templates;


//#region Models

#[derive(Serialize)]
struct Camera
{
    name: String,
    id: String,
}

//#endregion

fn index(templates: Data<Templates>) -> impl Responder
{
    templates.render("index.html", Context::new())
}

fn admin(templates: Data<Templates>) -> impl Responder
{
    let mut context = Context::new();

    // TODO: get from db
    context.insert("cameras", &[
        Camera {
            name: "Living Room".into(),
            id: "living-room".into(),
        },
        Camera {
            name: "Bedroom".into(),
            id: "bedroom".into(),
        },
    ]);

    templates.render("admin.html", context)
}

fn main()
{
    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    let mut hotwatch = Hotwatch::new()
        .expect("main: failed to initialize Hotwatch");
    let templates = Data::new(Templates::load(&mut hotwatch));
    let static_dir = std::env::var("LC_STATIC")
        .expect("main: could not read LC_STATIC");

    HttpServer::new(move || {
            App::new()
                .register_data(templates.clone())
                .service(web::scope("/api").configure(api::configure))
                .service(Files::new("/static", &static_dir))
                .route("/", web::get().to(index))
                .route("/admin/", web::get().to(admin))
        })
        .bind("127.0.0.1:8000").unwrap()
        .run().unwrap()
}
