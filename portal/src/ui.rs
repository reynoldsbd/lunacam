//! User interface

use actix_web::{Responder};
use actix_web::web::{self, Data, Path, ServiceConfig};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use log::{error};
use tera::{Context};
use crate::camera::Camera;
use crate::templates::Templates;


fn index(db: Data<SqliteConnection>, templates: Data<Templates>) -> impl Responder
{
    let mut context = Context::new();
    context.insert(
        "cameras",
        &Camera::get_all(&db)
            .expect("camera_admin: failed to get camera list")
    );

    templates.render("index.html", context)
}


fn camera(path: Path<(i32,)>, db: Data<SqliteConnection>, templates: Data<Templates>) -> impl Responder {

    let mut context = Context::new();
    context.insert(
        "camera",
        &Camera::get(path.0, &db)
            .expect("camera: failed to get specified camera")
    );

    templates.render("camera.html", context)
}


fn camera_admin(db: Data<SqliteConnection>, templates: Data<Templates>) -> impl Responder
{
    let mut context = Context::new();
    context.insert(
        "cameras",
        &Camera::get_all(&db)
            .expect("camera_admin: failed to get camera list")
    );

    templates.render("admin/cameras.html", context)
}


/// Configures an Actix service to serve the UI
pub fn configure(templates: Templates, db_url: String) -> impl FnOnce(&mut ServiceConfig)
{
    move |service| {
        service.data(templates);

        match SqliteConnection::establish(&db_url) {
            Ok(conn) => { service.data(conn); },
            Err(err) => error!("Failed to connect to database: {}", err),
        }

        service.route("/", web::get().to(index));
        service.route("/cameras/{id}", web::get().to(camera));
        service.route("/admin/cameras", web::get().to(camera_admin));
    }
}
