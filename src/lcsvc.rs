use std::env;
use std::mem;
use std::sync::RwLock;

use actix_files::Files;
use actix_web::{App, HttpServer};
use actix_web::web::{self, Data};
use reqwest::Client;
use tera::Tera;

use lunacam::cameras;
use lunacam::db;
use lunacam::error::Result;
use lunacam::logging;
use lunacam::stream;
use lunacam::ui;
use lunacam::users;


#[cfg(not(any(feature = "portal", feature = "stream-api")))]
compile_error!("invalid feature selection");


fn main() -> Result<()> {

    logging::init();

    // Prepare application resources

    #[cfg(debug_assertions)]
    let static_dir = env::var("LC_STATIC")?;

    let template_dir = env::var("LC_TEMPLATES")?;
    let template_dir = format!("{}/**/*", template_dir);
    let templates = Tera::new(&template_dir)?;

    let pool = db::connect()?;
    let conn = pool.get()?;

    #[cfg(feature = "portal")]
    cameras::initialize_proxy_config(&conn, &templates)?;
    
    #[cfg(feature = "portal")]
    users::maybe_create_default_user(&conn)?;

    #[cfg(feature = "stream")]
    let stream = Data::new(RwLock::new(stream::initialize(&conn)?));

    mem::drop(conn);

    let pool = Data::new(pool);
    let templates = Data::new(templates);
    let client = Data::new(Client::new());

    HttpServer::new(move || {

            let app = App::new()
                .register_data(pool.clone())
                .register_data(templates.clone())
                .register_data(client.clone());

            #[cfg(feature = "stream")]
            let app = app.register_data(stream.clone());

            let api = web::scope("api");
            #[cfg(feature = "portal")]
            let api = api.configure(cameras::configure_api)
                .configure(users::configure_api);
            #[cfg(feature = "stream-api")]
            let api = api.configure(stream::configure_api);
            let app = app.service(api);

            #[cfg(debug_assertions)]
            let app = app.service(Files::new("/static", &static_dir));

            #[cfg(feature = "portal")]
            let app = app.configure(ui::configure);

            app
        })
        .bind("127.0.0.1:9351")?
        .run()?;

    Ok(())
}
