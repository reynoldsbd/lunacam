//! User interface

use std::sync::Arc;
use actix_web::{Responder};
use actix_web::web::{self, Data, Path, ServiceConfig};
use diesel::r2d2::PoolError;
use tera::{Context};
use crate::{ConnectionPool, PooledConnection};
use crate::camera::CameraManager;
use crate::templates::{TemplateCollection};


struct UiResources {
    pool: ConnectionPool,
    templates: Arc<dyn TemplateCollection>,
}

impl CameraManager for UiResources {
    fn get_connection(&self) -> Result<PooledConnection, PoolError> {
        self.pool.get()
    }
}


fn index(resources: Data<UiResources>) -> impl Responder {

    let cameras = resources.get_cameras()
        .unwrap();

    let mut context = Context::new();
    context.insert("cameras", &cameras);

    resources.templates.response("index.html", context)
        .unwrap()
}


fn camera(path: Path<(i32,)>, resources: Data<UiResources>) -> impl Responder {

    let camera = resources.get_camera(path.0)
        .unwrap();

    let mut context = Context::new();
    context.insert("camera", &camera);

    resources.templates.response("camera.html", context)
        .unwrap()
}


fn camera_admin(resources: Data<UiResources>) -> impl Responder {

    let cameras = resources.get_cameras()
        .unwrap();

    let mut context = Context::new();
    context.insert("cameras", &cameras);

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
