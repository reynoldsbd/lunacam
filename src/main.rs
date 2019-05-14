mod auth;
mod config;
mod templates;


//#region Usings

use std::env;

use actix::{System, SystemRunner};

use actix_web::App;
use actix_web::fs::StaticFiles;
use actix_web::middleware::Logger;
use actix_web::middleware::session::{SessionStorage, CookieSessionBackend};
use actix_web::pred;
use actix_web::server::HttpServer;

use env_logger::Env;

use log::{debug, error, info, trace};

use config::SystemConfig;

//#endregion


/// Creates an App factory method for actix-web
fn make_app_factory(config: &SystemConfig) -> impl Fn() -> App + Clone {
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


//#region Actix System

fn sys_init(config: &SystemConfig) -> SystemRunner {
    let runner = System::new("lunacam");

    trace!("initializing HTTP server");
    HttpServer::new(make_app_factory(&config))
        .bind(&config.listen)
        .expect("could not bind to address")
        .start();

    trace!("registering termination handler");
    let system = System::current();
    ctrlc::set_handler(move || sys_term(&system))
        .unwrap_or_else(|e| error!("failed to register termination handler ({})", e));

    runner
}

fn sys_term(system: &System) {
    debug!("received termination signal");
    system.stop();
}

//#endregion


fn main() {
    let env = Env::default()
        .default_filter_or("info");
    env_logger::init_from_env(env);

    debug!("loading configuration");
    let args: Vec<_> = env::args().collect();
    let config = SystemConfig::load(&args[1])
        .expect("failed to load configuration");

    debug!("initializing system");
    let runner = sys_init(&config);
    info!("initialization complete");

    let status = runner.run();
    debug!("system exited with status {}", status);
}
