//! Serves the LunaCam API


// TODO: error handling


//#region Usings

use std::sync::Arc;

use actix::System;

use actix_web::{HttpRequest, HttpResponse, Json, Scope};

use log::{debug, info, trace};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::sec;
use crate::sec::{AccessLevel, Secrets};
use crate::stream::StreamManager;

//#endregion


//#region Actix application

/// Application state for the API
struct ApiState
{
    secrets: Config<Secrets>,
    smgr: Arc<StreamManager>,
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

        // Old session key is still being used by Actix middleware pipeline. Ideally, we would just
        // ask the HTTP server to reload itself, but there isn't really a clean way to do that from
        // this context (relevant PR: https://github.com/actix/actix-net/pull/20).
        //
        // As a workaround, we simply end the current process and allow it to be restarted by
        // systemd.
        crate::sys_term(&System::current());

        HttpResponse::Ok()
            .finish()
    }
}

/// Structure of the */admin/stream* resource
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamResource
{
    enabled: Option<bool>,
}

impl From<&StreamManager> for StreamResource
{
    fn from(smgr: &StreamManager) -> Self
    {
        let config = smgr.config.read();

        StreamResource {
            enabled: Some(config.enabled),
        }
    }
}

fn get_admin_stream() -> impl Fn(&HttpRequest<ApiState>) -> Json<StreamResource>
{
    |request| {
        Json(StreamResource::from(request.state().smgr.as_ref()))
    }
}

/// Handles *PATCH /admin/stream*
///
/// Reconfigures the video stream as directed by the user
fn patch_admin_stream() -> impl Fn(HttpRequest<ApiState>, Json<StreamResource>) -> HttpResponse
{
    |request, stream| {
        trace!("patch stream payload: {:?}", stream);

        let smgr = &request.state().smgr;

        if let Some(enabled) = stream.enabled {
            debug!("setting stream enabled status to {}", enabled);
            smgr.set_enabled(enabled);
        }

        HttpResponse::Ok()
            .finish()
    }
}

/// Configures LunaCam's API scope
pub fn scope(
    smgr: Arc<StreamManager>,
    secrets: Config<Secrets>
) -> impl FnOnce(Scope<()>) -> Scope<()>
{
    |scope| {
        trace!("configuring API scope");

        let state = ApiState {
            secrets: secrets,
            smgr: smgr,
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
                    r.get().f(get_admin_stream());
                    r.patch().with(patch_admin_stream());
                })
        })
    }
}

//#endregion
