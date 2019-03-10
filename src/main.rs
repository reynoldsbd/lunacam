use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use actix_web::App;
use actix_web::fs::StaticFiles;
use actix_web::middleware::Logger;
use actix_web::pred;
use actix_web::server;

use serde::Deserialize;


mod auth;
mod templates;


/// Service configuration values
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Configuration {
    listen: String,
    template_path: String,
    static_path: String,
    user_password: String,
    admin_password: String,
}

impl Configuration {

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Configuration, Box<Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}


fn make_app_factory(config: &Configuration) -> impl Fn() -> App + Clone {
    let templates = templates::TemplateManager::new(&config.template_path);
    let static_path = config.static_path.to_owned();
    let user_pw = config.user_password.to_owned();
    let admin_pw = config.admin_password.to_owned();

    move || {
        let templates = templates.clone();
        let user_pw = user_pw.clone();
        let admin_pw = admin_pw.clone();

        App::new()
            .middleware(Logger::default())
            .handler(
                "/static",
                StaticFiles::new(&static_path)
                    .expect("failed to load static file handler")
            )
            .resource("/{tail:.*}", move |r| {
                auth::LoginHandler::register(r, user_pw.to_owned(), admin_pw.to_owned());
                r.route()
                    .filter(pred::Get())
                    .h(templates.clone());
            })
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
