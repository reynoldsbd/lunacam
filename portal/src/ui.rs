//! User interface

use std::env;
use actix_web::{Responder};
use actix_web::web::{self, Data, ServiceConfig};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use tera::{Context};
use crate::camera::Camera;
use crate::templates::Templates;


fn index(templates: Data<Templates>) -> impl Responder
{
    templates.render("index.html", Context::new())
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
pub fn configure(templates: Templates) -> impl FnOnce(&mut ServiceConfig)
{
    move |service| {
            let database_url = unwrap_or_return!(env::var("DATABASE_URL"));
            let connection = unwrap_or_return!(SqliteConnection::establish(&database_url));
            service.data(connection);

            service.data(templates);

            service.route("/", web::get().to(index));
            service.route("/admin/cameras", web::get().to(camera_admin));
    }
}
