use std::env::{self, VarError};
use std::mem;
use std::sync::RwLock;

use actix_files::Files;
use actix_web::{App, HttpServer};
use actix_web::web::{self, Data};
use env_logger::Env;
use reqwest::Client;
use tera::Tera;

use lunacam::cameras;
use lunacam::db;
use lunacam::error::Result;
use lunacam::stream;
use lunacam::ui;
use lunacam::users;


#[cfg(not(any(feature = "portal", feature = "stream-api")))]
compile_error!("invalid feature selection");


#[cfg(debug_assertions)]
const DEFAULT_FILTER: &str = "info,lunacam=debug";
#[cfg(not(debug_assertions))]
const DEFAULT_FILTER: &str = "info";


/// Initializes environment-based logging provider
pub fn init_logging() {

    let env = Env::default()
        .filter_or("LC_LOG", DEFAULT_FILTER)
        .write_style("LC_LOG_STYLE");

    env_logger::init_from_env(env);
}


/// Loads templates from a directory on disk
///
/// Templates are loaded from the directory given by the LC_TEMPLATES
/// environment variable. If that variable is not present and this program is
/// compiled in debug mode, templates are loaded from *./templates*.
fn load_templates() -> Result<Tera> {

    let template_dir = match env::var("LC_TEMPLATES") {
        Ok(dir)                   => dir,
        #[cfg(debug_assertions)]
        Err(VarError::NotPresent) => String::from("templates"),
        Err(err)                  => return Err(err.into()),
    };

    let template_dir = format!("{}/**/*", template_dir);

    Ok(Tera::new(&template_dir)?)
}


fn main() -> Result<()> {

    init_logging();

    let client    = Data::new(Client::new());
    let templates = Data::new(load_templates()?);
    let pool      = Data::new(db::connect()?);

    // Perform initialization requiring database access
    let conn = pool.get()?;

    if cfg!(feature = "portal") {
        cameras::initialize_proxy_config(&conn, &templates)?;
        users::maybe_create_default_user(&conn)?;
    }

    #[cfg(feature = "stream")]
    let stream = Data::new(RwLock::new(stream::initialize(&conn)?));

    // Finished performing initialization requiring database access
    mem::drop(conn);

    HttpServer::new(move || {

            let app = App::new()
                .register_data(client.clone())
                .register_data(templates.clone())
                .register_data(pool.clone());

            #[cfg(feature = "stream")]
            let app = app.register_data(stream.clone());

            let api = web::scope("api");
            #[cfg(feature = "portal")]
            let api = api
                .configure(cameras::configure_api)
                .configure(users::configure_api);
            #[cfg(feature = "stream-api")]
            let api = api.configure(stream::configure_api);
            let app = app.service(api);

            #[cfg(debug_assertions)]
            let app = app
                .service(Files::new("/static/js",  "client/js"))
                .service(Files::new("/static/css", "build/css"));

            #[cfg(feature = "portal")]
            let app = app.configure(ui::configure);

            app
        })
        .bind("127.0.0.1:9351")?
        .run()?;

    Ok(())
}
