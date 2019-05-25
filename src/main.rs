// TODO: how to enable resetting secret?
// TODO: actors probably being used overzealously
// TODO: better capitalization for logged messages at info or above
// TODO: curly brace convention

mod api;
mod config;
mod sec;
mod templates;
mod ui;


//#region Usings

use std::env;
use std::sync::Arc;

use actix::{System, SystemRunner};

use actix_web::App;
use actix_web::fs::{StaticFiles};
use actix_web::middleware::{Logger};
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::server::HttpServer;

use env_logger::Env;

use log::{debug, error, info, trace};

use tera::compile_templates;

use crate::sec::{Secrets};
use crate::config::{Config, SystemConfig};

//#endregion


//#region Actix System

/// Returns an application factory callback
fn app_factory(config: SystemConfig) -> impl Fn() -> App + Clone + Send
{
    let config = Arc::new(config);
    let secrets: Config<Secrets> = Config::new("secrets")
        .expect("Failed to initialize secrets");
    let templates = Arc::new(compile_templates!(&format!("{}/**/*", &config.template_path)));

    move || {
        let static_files = StaticFiles::new(&config.static_path)
            .expect("Could not load static files");

        App::new()
            .middleware(Logger::default())
            .middleware(SessionStorage::new(
                CookieSessionBackend::private(&secrets.read().session_key)
                    .name("lc-session")
                    .secure(false)
            ))
            .handler("/static", static_files)
            .scope("/api", api::scope(secrets.clone()))
            .scope("", ui::scope(secrets.clone(), templates.clone()))
    }
}

/// Initializes the main Actix system
fn sys_init(config: SystemConfig) -> SystemRunner
{
    let runner = System::new("lunacam");

    // TODO: if config changes, restart dependent actors

    trace!("initializing HTTP server");
    let addr = config.listen.clone();
    HttpServer::new(app_factory(config))
        .bind(addr)
        .expect("Could not bind to address")
        .start();

    trace!("registering termination handler");
    let system = System::current();
    ctrlc::set_handler(move || sys_term(&system))
        .unwrap_or_else(|e| error!("Failed to register termination handler: {}", e));

    runner
}

/// Terminates the specified Actix system
///
/// This function is called to handle termination signals on all platforms. At present, it's only
/// responsibility is to stop the main Actix system, but in the future we should use it to manage
/// components that require graceful teardown.
fn sys_term(system: &System)
{
    debug!("received termination signal");
    system.stop();
}

//#endregion


fn main()
{
    let env = Env::default()
        .filter_or("LUNACAM_LOG", "info")
        .write_style("LUNACAM_LOG_STYLE");
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
