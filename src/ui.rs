//! Serves the LunaCam user interface


// TODO: error handling


//#region Usings

use actix_web::{Form, HttpRequest, HttpResponse, Scope};
use actix_web::http::header::LOCATION;

use log::{error, trace, warn};

use serde::{Deserialize};

use crate::config::Config;
use crate::sec;
use crate::sec::{AccessLevel, Secrets};
use crate::tmpl;
use crate::tmpl::Templates;

//#endregion


//#region Login

/// Contents of a POSTed login page form
#[derive(Deserialize)]
struct LoginForm
{
    password: String,
}

/// Handles *POST /login/*
///
/// Checks the user's password and authenticates the current session.
fn post_login() -> impl Fn(HttpRequest<UiState>, Form<LoginForm>) -> HttpResponse
{
    move |request, form| {
        trace!("logging in user");

        if sec::authenticate(&request, &form.password) {
            // TODO: this is hideous
            let dest = request.query()
                .get("dest")
                .map(|dest| dest.to_owned())
                .unwrap_or_else(||
                    request.url_for_static(RES_HOME)
                        .map(|url| url.as_str().to_owned())
                        .unwrap_or_else(|err| {
                            error!("Reverse-lookup of home resource failed: {}", err);
                            "/".to_owned()
                        })
                );

            HttpResponse::Found()
                .header(LOCATION, dest)
                .finish()

        } else {
            // TODO: display user-visible warning
            tmpl::render("login.html")(&request)
        }
    }
}

/// Creates an `HttpResponse` redirecting the user to the login page
///
/// Sets the `dest` query parameter to the current request's path, allowing `post_login` to
/// redirect back after a successful login.
fn login_redirect(request: &HttpRequest<UiState>) -> HttpResponse
{
    trace!("preparing login redirect");

    let mut url = request.url_for_static(RES_LOGIN)
        .expect("Reverse-lookup of login resource failed");

    request.uri()
        .path_and_query()
        .map(|dest| url.set_query(Some(&format!("dest={}", dest.as_str()))))
        .unwrap_or_else(|| warn!("Failed to set redirect destination"));

    HttpResponse::Found()
        .header(LOCATION, url.as_str())
        .finish()
}

//#endregion


//#region Actix Application

/// Application state for the UI
struct UiState
{
    secrets: Config<Secrets>,
    tmpl: Templates
}

impl AsRef<Config<Secrets>> for UiState
{
    fn as_ref(&self) -> &Config<Secrets>
    {
        &self.secrets
    }
}

impl AsRef<Templates> for UiState
{
    fn as_ref(&self) -> &Templates
    {
        &self.tmpl
    }
}

// Unique resource names
const RES_ADMIN: &str = "admin";
const RES_HOME: &str = "home";
const RES_LOGIN: &str = "login";

/// Configures LunaCam's UI scope
pub fn scope(secrets: Config<Secrets>, tmpl: Templates) -> impl FnOnce(Scope<()>) -> Scope<()>
{
    move |scope| {
        trace!("configuring UI scope");

        let state = UiState {
            secrets: secrets,
            tmpl: tmpl,
        };

        scope.with_state("", state, |scope| {
            scope
                .resource("/", |r| {
                    r.name(RES_HOME);
                    r.middleware(sec::require(AccessLevel::User, login_redirect));
                    r.get().f(tmpl::render("home.html"));
                })
                .resource("/login/", |r| {
                    r.name(RES_LOGIN);
                    r.get().f(tmpl::render("login.html"));
                    r.post().with(post_login());
                })
                .resource("/admin/", |r| {
                    r.name(RES_ADMIN);
                    r.middleware(sec::require(AccessLevel::Administrator, login_redirect));
                    r.get().f(tmpl::render("admin.html"));
                })
        })
    }
}

//#endregion
