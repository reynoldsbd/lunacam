//! Serves the LunaCam API


// TODO: error handling


//#region Usings

use actix_web::{App, HttpRequest, HttpResponse, Json};
use actix_web::dev::{Resource};
use actix_web::middleware::{Logger};
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};

use log::{debug, trace};

use serde::Deserialize;

use crate::config::Config;
use crate::ui::Secrets;

//#endrgegion


//#region Stream Control

#[derive(Deserialize)]
struct Stream {
    enabled: Option<bool>,
}

fn post_admin_stream() -> impl Fn(HttpRequest<()>, Json<Stream>) -> HttpResponse
{
    |request, stream| {
        if let Some(enabled) = stream.enabled {
            debug!("setting stream enabled status to {}", enabled);
            // TODO: smgr.set_enabled(stream.enabled)
        }

        HttpResponse::Ok()
            .finish()
    }
}

//#endregion


//#region Actix application

/// Configures the admin API's stream resource
fn res_admin_stream() -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.post().with(post_admin_stream());
    }
}

/// Returns an Actix application that provides LunaCam's web API
pub fn app(secrets: Config<Secrets>) -> App
{
    trace!("initializing API application");

    let app = App::new()
        .middleware(Logger::default())
        .middleware(SessionStorage::new(
            CookieSessionBackend::private(&secrets.read().session_key)
                .name("lc-session")
                .secure(false)
        ))
        .resource("/admin/stream", res_admin_stream());

    app
}

//#endregion
