//! User interface

use actix_web::HttpResponse;
use actix_web::web::{self, Data, Path, ServiceConfig};
use tera::{Context, Tera};

use lunacam::cameras;
use lunacam::db::{ConnectionPool};
use lunacam::error::Result;
use lunacam::users;


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


fn index(pool: Data<ConnectionPool>, templates: Data<Tera>) -> Result<HttpResponse> {

    let conn = pool.get()?;
    let cameras = cameras::all(&conn)?;

    let mut context = Context::new();
    context.insert("cameras", &cameras);

    render_template_response(&templates, "index.html", context)
}


fn camera(
    pool: Data<ConnectionPool>,
    templates: Data<Tera>,
    path: Path<(i32,)>,
) -> Result<HttpResponse>
{
    let conn = pool.get()?;
    let camera = cameras::get(path.0, &conn)?;

    let mut context = Context::new();
    context.insert("camera", &camera);

    render_template_response(&templates, "camera.html", context)
}


fn camera_admin(pool: Data<ConnectionPool>, templates: Data<Tera>) -> Result<HttpResponse> {

    let conn = pool.get()?;
    let cameras = cameras::all(&conn)?;

    let mut context = Context::new();
    context.insert("cameras", &cameras);

    render_template_response(&templates, "admin/cameras.html", context)
}


fn user_admin(pool: Data<ConnectionPool>, templates: Data<Tera>) -> Result<HttpResponse> {

    let conn = pool.get()?;
    let users = users::all(&conn)?;

    let mut context = Context::new();
    context.insert("users", &users);

    render_template_response(&templates, "admin/users.html", context)
}


/// Configures an Actix service to serve the UI
pub fn configure(service: &mut ServiceConfig) {

    service.route("/", web::get().to(index));
    service.route("/cameras/{id}", web::get().to(camera));
    service.route("/admin/cameras", web::get().to(camera_admin));
    service.route("/admin/users", web::get().to(user_admin));
}
