use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse};
use actix_web::dev::Handler;
use actix_web::http::{Method, StatusCode};

use tera::{compile_templates, Context, Tera};


/// Manages HTML templates
///
/// TemplateManager maintains a logical collection of Tera templates and can act as an Actix Handler
/// that renders those templates when a corresponding URL is requested.
#[derive(Clone)]
pub struct TemplateManager(Arc<Tera>);

impl TemplateManager {

    /// Returns a new TemplateManager
    ///
    /// # TODOs
    ///
    /// * Would accepting something other than &str (like AsRef) make construction easier?
    /// * Once Tera hits 1.0, it will be safe to propagate Tera errors
    pub fn new(path: &str) -> TemplateManager {
        TemplateManager(Arc::new(compile_templates!(&format!("{}/**/*", path))))
    }
}

impl<S> Handler<S> for TemplateManager {
    type Result = HttpResponse;
    fn handle(&self, request: &HttpRequest<S>) -> Self::Result {

        // Only makes sense to render for GET
        if request.method() != Method::GET {
            return HttpResponse::new(StatusCode::NOT_FOUND);
        }

        // Extract requested path
        let mut name: String = match request.match_info().query::<String>("tail") {
            // Skip the leading "/"
            Ok(tail) => tail.chars().skip(1).collect(),
            // Missing tail means this handler wasn't properly registered with the Actix App
            Err(_) => return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
        };

        // Map nice-looking paths to actual template files
        if name == "" {
            // "/" -> "index.html"
            name = "index.html".to_owned();
        } else if name.ends_with("/") {
            // "/foo/" -> "/foo/index.html"
            name += "index.html";
        } else {
            // "/foo" -> "/foo.html"
            name += ".html";
        }

        let content = match self.0.render(&name, &Context::new()) {
            Ok(content) => content,
            Err(_) => return HttpResponse::new(StatusCode::NOT_FOUND),
        };

        HttpResponse::Ok()
            .content_type("text/html")
            .body(content)
    }
}
