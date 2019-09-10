//! User interface

use std::sync::Arc;
use actix_web::HttpResponse;
use actix_web::web::{self, Data, Path, ServiceConfig};
use lunacam::Result;
use tera::{Context};
use lunacam::db::{ConnectionPool, PooledConnection};
use crate::camera::CameraManager;
use crate::templates::{TemplateCollection};


struct UiResources {
    pool: ConnectionPool,
    templates: Arc<dyn TemplateCollection>,
}

impl CameraManager for UiResources {
    fn get_connection(&self) -> Result<PooledConnection> {
        Ok(self.pool.get()?)
    }
}


fn index(resources: Data<UiResources>) -> Result<HttpResponse> {

    let cameras = resources.get_cameras()?;

    let mut context = Context::new();
    context.insert("cameras", &cameras);

    resources.templates.response("index.html", context)
}


fn camera(path: Path<(i32,)>, resources: Data<UiResources>) -> Result<HttpResponse> {

    let camera = resources.get_camera(path.0)?;

    let mut context = Context::new();
    context.insert("camera", &camera);

    resources.templates.response("camera.html", context)
}


fn camera_admin(resources: Data<UiResources>) -> Result<HttpResponse> {

    let cameras = resources.get_cameras()?;

    let mut context = Context::new();
    context.insert("cameras", &cameras);

    resources.templates.response("admin/cameras.html", context)
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
