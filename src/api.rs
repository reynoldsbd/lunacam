//! Serves the LunaCam API


// TODO: error handling


//#region Usings

use actix_web::{HttpRequest, HttpResponse, Json, Scope};

use log::{debug, info, trace};

use serde::Deserialize;

use crate::config::Config;
use crate::sec;
use crate::sec::{AccessLevel, Secrets};

//#endrgegion


//#region Actix application

/// Application state for the API
struct ApiState
{
    secrets: Config<Secrets>,
}

/// Structure of the */admin/passwords* resource
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordPatch
{
    admin_pw: Option<String>,
    user_pw: Option<String>,
}

/// Handles *PATCH /admin/passwords*
///
/// Updates user and admin passwords as directed by user
fn patch_admin_passwords() -> impl Fn(HttpRequest<ApiState>, Json<PasswordPatch>) -> HttpResponse
{
    |request, passwords| {
        trace!("updating passwords");

        let mut secrets = request.state()
            .secrets
            .write();

        if let Some(ref pw) = passwords.admin_pw {
            info!("Updating admin password");
            secrets.admin_pw = pw.to_owned();
        }

        if let Some(ref pw) = passwords.user_pw {
            info!("Updating user password");
            secrets.user_pw = pw.to_owned();
        }

        HttpResponse::Ok()
            .finish()
    }
}

/// Handles *DELETE /admin/sessions*
///
/// Resets the session key, which effectively forcing all users to re-authenticate.
fn delete_admin_sessions() -> impl Fn(&HttpRequest<ApiState>) -> HttpResponse
{
    |request| {
        info!("Resetting all login sessions");
        request.state()
            .secrets
            .write()
            .reset_session_key();
        // TODO: only reset the HTTP server (https://github.com/actix/actix-net/pull/20)

        HttpResponse::Ok()
            .finish()
    }
}

/// Structure of the */admin/stream* resource
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamPatch
{
    enabled: Option<bool>,
}

/// Handles *PATCH /admin/stream*
///
/// Reconfigures the video stream as directed by the user
fn patch_admin_stream() -> impl Fn(Json<StreamPatch>) -> HttpResponse
{
    |stream| {
        trace!("configuring video stream");

        if let Some(enabled) = stream.enabled {
            debug!("setting stream enabled status to {}", enabled);
            // TODO: smgr.set_enabled(stream.enabled)
        }

        HttpResponse::Ok()
            .finish()
    }
}

/// Configures LunaCam's API scope
pub fn scope(secrets: Config<Secrets>) -> impl FnOnce(Scope<()>) -> Scope<()>
{
    |scope| {
        trace!("configuring API scope");

        let state = ApiState {
            secrets: secrets,
        };

        scope.with_state("", state, |scope| {
            scope
                // This makes all API resources admin-only. May need to fall back to per-resource
                // middleware if we introduce any non-admin API resources.
                .middleware(sec::require(
                    AccessLevel::Administrator,
                    |_| HttpResponse::Unauthorized().finish()
                ))
                .resource("/admin/passwords", |r| {
                    r.patch().with(patch_admin_passwords());
                })
                .resource("/admin/sessions", |r| {
                    r.delete().f(delete_admin_sessions());
                })
                .resource("/admin/stream", |r| {
                    r.patch().with(patch_admin_stream());
                })
        })
    }
}

//#endregion
