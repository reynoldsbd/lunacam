use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use actix_web::App;
use actix_web::fs::StaticFiles;
use actix_web::middleware::Logger;
use actix_web::middleware::session::{SessionStorage, CookieSessionBackend};
use actix_web::pred;
use actix_web::server;

use base64::STANDARD;

use base64_serde::base64_serde_type;

use serde::Deserialize;


mod auth;
mod templates;


/// Provides bas64 encoding/decoding for the configuration system
base64_serde_type!(BASE64, STANDARD);


/// Service configuration values
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Configuration {

    /// Address on which LunaCam web service listens
    listen: String,

    /// Path to HTML templates
    template_path: String,

    /// Path to static content (CSS, etc.)
    static_path: String,

    /// Password required for user-level access
    user_password: String,

    /// Password required for administrative access
    admin_password: String,

    /// Key used to encrypt session cookies (must be 32 bytes, base64-encoded)
    #[serde(with = "BASE64")]
    secret: Vec<u8>,
}

impl Configuration {

    /// Loads Configuration from the specified file
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Configuration, Box<Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}


/// Creates an App factory method for actix-web
fn make_app_factory(config: &Configuration) -> impl Fn() -> App + Clone {
    let templates = templates::TemplateManager::new(&config.template_path);
    let static_path = config.static_path.to_owned();
    let user_pw = config.user_password.to_owned();
    let admin_pw = config.admin_password.to_owned();
    let secret = config.secret.clone();

    move || {
        let templates = templates.clone();
        let user_pw = user_pw.clone();
        let admin_pw = admin_pw.clone();
        let secret = secret.clone();

        App::new()
            .middleware(Logger::default())
            .middleware(SessionStorage::new(
                CookieSessionBackend::private(&secret)
                    .name("lunacamsession")
                .secure(false) // TODO: is there a way to enable secure cookies?
            ))
            .handler(
                "/static",
                StaticFiles::new(&static_path)
                    .expect("failed to load static file handler")
            )
            .resource("/{tail:.*}", move |r| {
                auth::Authenticator::register(r, user_pw.to_owned(), admin_pw.to_owned());
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
