//! Security and access control
//!
//! # Authentication and Authorization
//!
//! This module provides tools for controlling access to application resources. These tools use
//! Actix's session storage to track the login status and level of access of each request.
//!
//! Use `require` to prevent unauthorized resource access. This function returns an instance of
//! `Middleware`, which can be applied to individual resources, application scopes, or even entire
//! applications.
//!
//! Requests are unauthenticated by default (their sessions are not stamped with an access level).
//! Use `authenticate` to validate a password and stamp the current session with an appropriate
//! access level.
//!
//! # Secrets
//!
//! Encryption and authentication secrets are stored in a `Secrets`, which in turn is stored using
//! `Config`. This implementation detail is intentionally made public to allow administrators, via
//! the admin API, to change passwords or reset sessions.


//#region Usings

use actix_web::{HttpRequest, HttpResponse};
use actix_web::error;
use actix_web::middleware::{Middleware, Started};
use actix_web::middleware::session::RequestSession;

use derive_more::Display;

use log::{debug, error, trace, warn};

use rand::Rng;

use serde::{Deserialize, Serialize};

use crate::config::Config;

//#endregion


//#region Secrets

/// Secret values used for authentication and session encryption
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Secrets
{
    pub admin_pw: String,
    pub session_key: [u8; 32],
    pub user_pw: String,
}

impl Secrets
{
    /// Resets the current session key to a new, random value
    ///
    /// Caller is responsible for ensuring the HTTP server is restarted (otherwise, the old key will
    /// continue to be used).
    pub fn reset_session_key(&mut self)
    {
        self.session_key = rand::thread_rng().gen();
    }
}

/// Ensures that `session_key` always has a random, secure value
impl Default for Secrets
{
    fn default() -> Self
    {
        trace!("generating new secrets");
        Secrets {
            admin_pw: Default::default(),
            session_key: rand::thread_rng().gen(),
            user_pw: Default::default(),
        }
    }
}

//#endregion


//#region AccessLevel

/// Defines a user's access level
///
/// Access levels are sequential, and a higher level of access implies all lower levels.
#[derive(Deserialize, Display, PartialEq, PartialOrd, Serialize)]
pub enum AccessLevel
{
    // Ordering of variant declaration matters! This ordering is used by the derivation of
    // PartialOrd, which in turn is used to allow or deny resource access according to the precence
    // rules specified above.
    User,
    Administrator,
}

/// Name of secure cookie used to store access level
const ACCESS_LEVEL_COOKIE: &str = "accessLevel";

//#endregion


//#region Authenticator

/// Authenticates a user session
///
/// `password` is checked against expected user and administrative passwords, and if a match is
/// found the session associated with `request` is stamped with the appropriate access level.
pub fn authenticate<S>(request: &HttpRequest<S>, password: &str) -> bool
where S: AsRef<Config<Secrets>>
{
    trace!("authenticating request");

    let secrets = request.state()
        .as_ref()
        .read();

    let level = if password == secrets.user_pw {
        AccessLevel::User
    } else if password == secrets.admin_pw {
        AccessLevel::Administrator
    } else {
        warn!("Failed authentication attempt");
        return false;
    };

    debug!("authenticated request with access level {}", level);
    request.session()
        .set(ACCESS_LEVEL_COOKIE, level)
        .unwrap_or_else(|err| error!("Failed to set access level cookie: {}", err));
    true
}

//#endregion


//#region Authorization

/// Middleware that rejects unauthorized requests
struct AccessLevelMiddleware<R>
{
    min_level: AccessLevel,
    responder: R,
}

impl<R, S> Middleware<S> for AccessLevelMiddleware<R>
where R: Fn(&HttpRequest<S>) -> HttpResponse + 'static
{
    fn start(&self, request: &HttpRequest<S>) -> error::Result<Started>
    {
        let level = request.session()
            .get::<AccessLevel>(ACCESS_LEVEL_COOKIE)?
            .filter(|level| level >= &self.min_level);

        if let Some(level) = level {
            trace!("permitting request with access level {}", level);
            Ok(Started::Done)
        } else {
            debug!("rejecting request (access level missing or too low)");
            Ok(Started::Response((self.responder)(request)))
        }
    }
}

/// Returns middleware that rejects unauthorized requests
///
/// If the current session's access level is not present or lower than `min`, the request is
/// rejected and `reject` is called to produce a response.
pub fn require<R, S>(min: AccessLevel, reject: R) -> impl Middleware<S>
where R: Fn(&HttpRequest<S>) -> HttpResponse + 'static
{
    AccessLevelMiddleware {
        min_level: min,
        responder: reject,
    }
}

//#endregion
