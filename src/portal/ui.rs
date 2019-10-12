//! User interface

use actix_web::HttpResponse;
use actix_web::web::{self, Data, Path, ServiceConfig};
use lunacam::Result;
use tera::{Context, Tera};
use crate::Resources;
use crate::camera::CameraManager;


fn render_template_response(
    templates: &Tera,
    name: &str,
    context: Context,
) -> Result<HttpResponse> {

    let body = templates.render(name, context)?;

    let response = HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}


fn index(resources: Data<Resources>) -> Result<HttpResponse> {

    let cameras = resources.get_cameras()?;

    let mut context = Context::new();
    context.insert("cameras", &cameras);

    render_template_response(&resources.templates, "index.html", context)
}


fn camera(path: Path<(i32,)>, resources: Data<Resources>) -> Result<HttpResponse> {

    let camera = resources.get_camera(path.0)?;

    let mut context = Context::new();
    context.insert("camera", &camera);

    render_template_response(&resources.templates, "camera.html", context)
}


fn camera_admin(resources: Data<Resources>) -> Result<HttpResponse> {

    let cameras = resources.get_cameras()?;

    let mut context = Context::new();
    context.insert("cameras", &cameras);

    render_template_response(&resources.templates, "admin/cameras.html", context)
}


fn user_admin(resources: Data<Resources>) -> Result<HttpResponse> {

    let context = Context::new();

    render_template_response(&resources.templates, "admin/users.html", context)
}


/// Configures an Actix service to serve the UI
pub fn configure(service: &mut ServiceConfig) {

    service.route("/", web::get().to(index));
    service.route("/cameras/{id}", web::get().to(camera));
    service.route("/admin/cameras", web::get().to(camera_admin));
    service.route("/admin/users", web::get().to(user_admin));
}
