// TODO: how to enable resetting secret?
// TODO: actors probably being used overzealously
// TODO: better capitalization for logged messages at info or above

mod api;
mod auth;
mod config;
mod templates;
mod ui;


//#region Usings

use std::env;
use std::sync::Arc;

use actix::{Actor, System, SystemRunner};

use actix_web::App;
use actix_web::fs::StaticFiles;
use actix_web::middleware::Logger;
use actix_web::middleware::session::{SessionStorage, CookieSessionBackend};
use actix_web::pred;
use actix_web::server::HttpServer;

use env_logger::Env;

use log::{debug, error, info, trace};

use crate::auth::NewAuthenticator;
use crate::config::SystemConfig;

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

fn app_factory(config: SystemConfig) -> impl Fn() -> Vec<App> + Clone + Send {

    trace!("initializing authentication system");
    let auth = NewAuthenticator::new(&config)
        .expect("failed to initialize authentication system")
        .start();
    let config = Arc::new(config);

    move || {
        vec![
            ui::app(auth.clone(), &config),
            api::app()
                .prefix("/api"),
        ]
    }
}

fn sys_init(config: SystemConfig) -> SystemRunner {
    let runner = System::new("lunacam");

    // TODO: if config changes, restart dependent actors

    trace!("initializing HTTP server");
    let addr = config.listen.clone();
    HttpServer::new(app_factory(config))
        .bind(addr)
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
    let runner = sys_init(config);
    info!("initialization complete");

    let status = runner.run();
    debug!("system exited with status {}", status);
}
