//! User interface

use std::sync::Arc;
use actix_web::{Responder};
use actix_web::web::{self, Data, Path, ServiceConfig};
use tera::{Context};
use crate::{ConnectionPool};
use crate::camera::Camera;
use crate::templates::{TemplateCollection};


struct UiResources {
    pool: ConnectionPool,
    templates: Arc<dyn TemplateCollection>,
}


fn index(resources: Data<UiResources>) -> impl Responder
{
    let conn = resources.pool.get()
        .unwrap();

    let mut context = Context::new();
    context.insert(
        "cameras",
        &Camera::get_all(&conn)
            .expect("camera_admin: failed to get camera list")
    );

    resources.templates.response("index.html", context)
        .unwrap()
}


fn camera(path: Path<(i32,)>, resources: Data<UiResources>) -> impl Responder {

    let conn = resources.pool.get()
        .unwrap();

    let mut context = Context::new();
    context.insert(
        "camera",
        &Camera::get(path.0, &conn)
            .expect("camera: failed to get specified camera")
    );

    resources.templates.response("camera.html", context)
        .unwrap()
}


fn camera_admin(resources: Data<UiResources>) -> impl Responder
{
    let conn = resources.pool.get()
        .unwrap();

    let mut context = Context::new();
    context.insert(
        "cameras",
        &Camera::get_all(&conn)
            .expect("camera_admin: failed to get camera list")
    );

    resources.templates.response("admin/cameras.html", context)
        .unwrap()
}


/// Configures an Actix service to serve the UI
pub fn configure(templates: Arc<dyn TemplateCollection>, pool: ConnectionPool) -> impl FnOnce(&mut ServiceConfig)
{
    move |service| {
        service.data(UiResources {
            pool: pool,
            templates: templates,
        });

        service.route("/", web::get().to(index));
        service.route("/cameras/{id}", web::get().to(camera));
        service.route("/admin/cameras", web::get().to(camera_admin));
    }
}
