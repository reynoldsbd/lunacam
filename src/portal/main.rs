use std::env;
#[cfg(debug_assertions)]
use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self, Data, ServiceConfig};
use reqwest::Client;
use tera::Tera;

use lunacam::cameras;
use lunacam::db;
use lunacam::error::Result;
use lunacam::logging;
use lunacam::users;

mod ui;


/// Configures an Actix service to serve the API
fn configure_api(service: &mut ServiceConfig) {

    cameras::configure_api(service);
    users::configure_api(service);
}


fn main() -> Result<()> {

    logging::init();

    #[cfg(debug_assertions)]
    let static_dir = env::var("LC_STATIC")?;

    let pool = db::connect()?;

    let template_dir = env::var("LC_TEMPLATES")?;
    let template_dir = format!("{}/**/*", template_dir);
    let templates = Tera::new(&template_dir)?;

    {
        let conn = pool.get()?;
        cameras::initialize_proxy_config(&conn, &templates)?;
        users::maybe_create_default_user(&conn)?;
    }

    let pool = Data::new(pool);
    let templates = Data::new(templates);
    let client = Data::new(Client::new());

    HttpServer::new(move || {
            let app = App::new();
            let app = app.register_data(pool.clone());
            let app = app.register_data(templates.clone());
            let app = app.register_data(client.clone());
            let app = app.service(web::scope("/api").configure(configure_api));
            #[cfg(debug_assertions)]
            let app = app.service(Files::new("/static", &static_dir));
            let app = app.configure(ui::configure);

            app
        })
        .bind("127.0.0.1:9351")?
        .run()?;

    Ok(())
}
