#[macro_use]
extern crate diesel;

use std::env;
#[cfg(debug_assertions)]
use actix_files::{Files};
use actix_web::{App, HttpServer};
use actix_web::web::{self, Data};
use reqwest::Client;
use lunacam::Result;
use lunacam::db::{self, ConnectionPool};
use lunacam::logging;
use tera::Tera;

mod api;
mod camera;
mod ui;

use camera::CameraManager;


/// Application resources used to service requests
struct Resources {
    client: Client,
    pool: ConnectionPool,
    templates: Tera,
}

impl Resources {

    fn load() -> Result<Resources> {

        let pool = db::connect()?;

        let template_dir = env::var("LC_TEMPLATES")?;
        let template_dir = format!("{}/**/*", template_dir);
        let templates = Tera::new(&template_dir)?;

        Ok(Resources {
            client: Client::new(),
            pool,
            templates,
        })
    }
}

impl std::borrow::Borrow<ConnectionPool> for Resources {
    fn borrow(&self) -> &ConnectionPool {
        &self.pool
    }
}

impl AsRef<Tera> for Resources {
    fn as_ref(&self) -> &Tera {
        &self.templates
    }
}

impl AsRef<Client> for Resources {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}


fn main() -> Result<()> {

    logging::init();

    #[cfg(debug_assertions)]
    let static_dir = env::var("LC_STATIC")?;

    let resources = Resources::load()?;

    resources.initialize_proxy()?;

    let pool = Data::new(resources.pool.clone());
    let resources = Data::new(resources);

    HttpServer::new(move || {
            let app = App::new();
            let app = app.register_data(resources.clone());
            let app = app.register_data(pool.clone());
            let app = app.service(web::scope("/api").configure(api::configure));
            #[cfg(debug_assertions)]
            let app = app.service(Files::new("/static", &static_dir));
            let app = app.configure(ui::configure);

            app
        })
        .bind("127.0.0.1:9351")?
        .run()?;

    Ok(())
}
