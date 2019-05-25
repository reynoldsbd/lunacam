//! Serves the LunaCam user interface


// TODO: error handling


//#region Usings

use std::sync::Arc;

use actix_web::{Form, HttpRequest, HttpResponse, Scope};
use actix_web::http::header::LOCATION;

use log::{debug, error, trace, warn};

use serde::{Deserialize};

use tera::{Context, Tera};

use crate::config::Config;
use crate::sec;
use crate::sec::{AccessLevel, Secrets};

//#endregion


//#region Helpers

// TODO: use an actor to support reloading

/// Renders the specified template to an `HttpResponse`
///
/// If an error occurs while rendering the template, the error error message is logged and written
/// to the returned response.
fn render(name: &'static str) -> impl Fn(&HttpRequest<UiState>) -> HttpResponse + 'static
{
    move |request| {
        debug!("rendering template {}", name);
        request.state()
            .templates
            .render(name, &Context::new())
            .map(|body|
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(body)
            )
            .unwrap_or_else(|err| {
                error!("error rendering template: {}", err);
                HttpResponse::InternalServerError()
                    .body(format!("error rendering template: {}", err))
            })
    }
}

/// Creates an `HttpResponse` redirecting the user to the login page
///
/// Sets the `dest` query parameter to the current request's path, allowing `post_login` to
/// redirect back after a successful login.
fn login_redirect(request: &HttpRequest<UiState>) -> HttpResponse
{
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
    templates: Arc<Tera>,
}

impl AsRef<Config<Secrets>> for UiState
{
    fn as_ref(&self) -> &Config<Secrets>
    {
        &self.secrets
    }
}

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
            render("login.html")(&request)
        }
    }
}

// Unique resource names
const RES_ADMIN: &str = "admin";
const RES_HOME: &str = "home";
const RES_LOGIN: &str = "login";

/// Configures LunaCam's UI scope
pub fn scope(secrets: Config<Secrets>, templates: Arc<Tera>) -> impl FnOnce(Scope<()>) -> Scope<()>
{
    move |scope| {
        trace!("configuring UI scope");

        let state = UiState {
            secrets: secrets,
            templates: templates,
        };

        scope.with_state("", state, |scope| {
            scope
                .resource("/", |r| {
                    r.name(RES_HOME);
                    r.middleware(sec::require(AccessLevel::User, login_redirect));
                    r.get().f(render("home.html"));
                })
                .resource("/login/", |r| {
                    r.name(RES_LOGIN);
                    r.get().f(render("login.html"));
                    r.post().with(post_login());
                })
                .resource("/admin/", |r| {
                    r.name(RES_ADMIN);
                    r.middleware(sec::require(AccessLevel::Administrator, login_redirect));
                    r.get().f(render("admin.html"));
                })
        })
    }
}

//#endregion
