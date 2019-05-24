//! Serves the LunaCam API


// TODO: error handling


//#region Usings

use actix_web::{HttpResponse, Json, Scope};
use actix_web::dev::{Resource};

use log::{debug, trace};

use serde::Deserialize;

//#endrgegion


//#region Stream Control

#[derive(Deserialize)]
struct Stream {
    enabled: Option<bool>,
}

fn post_admin_stream() -> impl Fn(Json<Stream>) -> HttpResponse
{
    |stream| {
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

/// Configures LunaCam's API scope
pub fn scope() -> impl FnOnce(Scope<()>) -> Scope<()>
{
    |scope| {
        trace!("configuring API scope");

        // TODO: middleware to reject unauthenticated requests
        scope
            .resource("/admin/stream", res_admin_stream())
    }
}

//#endregion
