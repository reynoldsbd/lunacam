use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use actix_web::{App, HttpResponse};
use actix_web::dev::Resource;
use actix_web::error::ResponseError;
use actix_web::fs::StaticFiles;
use actix_web::http::StatusCode;
use actix_web::middleware::Logger;
use actix_web::server;

use failure::Fail;

use serde::Deserialize;

use tera::{compile_templates, Context, Tera};


/// Service configuration values
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Configuration {
    listen: String,
    template_path: String,
    static_path: String,
}

impl Configuration {

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Configuration, Box<Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}


/// Error returned when web operations fail
///
/// # TODO:
///
/// * seems unnecessarily complex, could this be simplified to a newtype?
/// * once tera 1.0 is released, we might be able to return the template error directly
#[derive(Debug, Fail)]
enum WebError {
    #[fail(display = "failed to load template\"{}\"", 0)]
    Template(String),
}

impl ResponseError for WebError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            WebError::Template(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}


/// Templates that can be rendered by actix
///
/// This wraps a Tera template collection and provides two key features:
///
/// * synchronization
/// * utility method that binds an actix resource to a particular template
#[derive(Clone)]
struct TemplateCollection(Arc<Tera>);

impl TemplateCollection {

    fn new(glob: &str) -> TemplateCollection {
        TemplateCollection(Arc::new(compile_templates!(glob)))
    }

    fn register(&self, template_name: &str) -> impl Fn(&mut Resource) {
        let templates = self.0.clone();
        let template_name = template_name.to_owned();

        move |res| {
            let templates = templates.clone();
            let template_name = template_name.to_owned();

            res.f(move |_req| {
                templates.render(&template_name, &Context::new())
                    .map(|content| {
                        HttpResponse::Ok()
                            .content_type("text/html")
                            .body(content)
                    })
                    .map_err(|_| WebError::Template(template_name.clone()))
            });
        }
    }
}


fn make_app_factory(config: &Configuration) -> impl Fn() -> App + Clone {
    let templates = TemplateCollection::new(&config.template_path);
    let static_path = config.static_path.to_owned();

    move || {
        App::new()
            .middleware(Logger::default())
            .resource("/", templates.register("index.html"))
            .resource("/login", templates.register("login.html"))
            .handler(
                "/static",
                StaticFiles::new(&static_path)
                    .expect("failed to load static file handler")
            )
    }
}


fn main() {
    let args: Vec<_> = env::args().collect();
    let config = Configuration::from_file(&args[1])
        .expect("failed to load configuration");

    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let app_factory = make_app_factory(&config);
    server::new(app_factory)
        .bind(&config.listen)
        .expect("could not bind to address")
        .run();
}
