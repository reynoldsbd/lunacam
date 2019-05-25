//! Serves the LunaCam API


// TODO: error handling


//#region Usings

use actix_web::{HttpResponse, Json, Scope};

use log::{debug, trace};

use serde::Deserialize;

use crate::sec;
use crate::sec::AccessLevel;

//#endrgegion


//#region Actix application

#[derive(Deserialize)]
struct Stream {
    enabled: Option<bool>,
}

fn patch_admin_stream() -> impl Fn(Json<Stream>) -> HttpResponse
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

/// Configures LunaCam's API scope
pub fn scope() -> impl FnOnce(Scope<()>) -> Scope<()>
{
    |scope| {
        trace!("configuring API scope");

        scope
            // This makes all API resources admin-only. May need to fall back to per-resource
            // middleware if we introduce any non-admin API resources.
            .middleware(sec::require(
                AccessLevel::Administrator,
                |_| HttpResponse::Unauthorized().finish()
            ))
            .resource("/admin/stream", |r| {
                r.patch().with(patch_admin_stream());
            })
    }
}

//#endregion
