//! User interface

use actix_web::{Responder};
use actix_web::web::{self, Data, Path, ServiceConfig};
use tera::{Context};
use crate::{ConnectionPool};
use crate::camera::Camera;
use crate::templates::Templates;


fn index(pool: Data<ConnectionPool>, templates: Data<Templates>) -> impl Responder
{
    let conn = pool.get()
        .unwrap();

    let mut context = Context::new();
    context.insert(
        "cameras",
        &Camera::get_all(&conn)
            .expect("camera_admin: failed to get camera list")
    );

    templates.render("index.html", context)
}


fn camera(path: Path<(i32,)>, pool: Data<ConnectionPool>, templates: Data<Templates>) -> impl Responder {

    let conn = pool.get()
        .unwrap();

    let mut context = Context::new();
    context.insert(
        "camera",
        &Camera::get(path.0, &conn)
            .expect("camera: failed to get specified camera")
    );

    templates.render("camera.html", context)
}


fn camera_admin(pool: Data<ConnectionPool>, templates: Data<Templates>) -> impl Responder
{
    let conn = pool.get()
        .unwrap();

    let mut context = Context::new();
    context.insert(
        "cameras",
        &Camera::get_all(&conn)
            .expect("camera_admin: failed to get camera list")
    );

    templates.render("admin/cameras.html", context)
}


/// Configures an Actix service to serve the UI
pub fn configure(templates: Templates, pool: ConnectionPool) -> impl FnOnce(&mut ServiceConfig)
{
    move |service| {
        service.data(templates);
        service.data(pool);

        service.route("/", web::get().to(index));
        service.route("/cameras/{id}", web::get().to(camera));
        service.route("/admin/cameras", web::get().to(camera_admin));
    }
}
