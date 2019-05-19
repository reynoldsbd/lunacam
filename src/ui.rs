//! Serves the LunaCam user interface


// TODO: error handling


//#region Usings

use std::sync::Arc;

use actix_web::{App, HttpRequest, HttpResponse};
use actix_web::dev::Resource;
use actix_web::fs::StaticFiles;
use actix_web::http::header::LOCATION;
use actix_web::middleware::Logger;
use actix_web::middleware::session::{CookieSessionBackend, RequestSession, SessionStorage};

use log::{debug, error, trace, warn};

use rand::Rng;

use serde::{Deserialize, Serialize};

use tera::{Context, Tera};

use crate::config::{Config, SystemConfig};

//#endregion


//#region Security

// TODO: Shouldn't need to be public. Could we use a singleton or Actix service?

/// Secret values used for authentication and session encryption
#[derive(Deserialize, Serialize)]
pub struct Secrets
{
    pub admin_pw: String,
    pub session_key: [u8; 32],
    pub user_pw: String,
}

/// Custom implementation of `Default` ensures that `session_key` always has a random, secure value
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


//#region Templates

/// Renders the specified template to an `HttpResponse`
///
/// If an error occurs while rendering the template, the error error message is logged and written
/// to the returned response.
fn render(templates: &Tera, name: &str) -> HttpResponse
{
    debug!("rendering template {}", name);
    templates.render(name, &Context::new())
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

//#endregion


//#region Authentication and Login Page

/// Defines a user's access level
///
/// Ordering of variant declarations matters! This ordering is used to derive `PartialOrd`
#[derive(Deserialize, PartialEq, PartialOrd, Serialize)]
enum AccessLevel
{
    Any,
    Administrative,
}

/// Name of secure cookie used to store access level
const ACCESS_LEVEL_COOKIE: &str = "accessLevel";

/// Validates user's access level
///
/// Checks the user's session for an access level, then returns whether the access level is equal to
/// or above the level specified by `min`. If an access level is not set, this function returns
/// `false`.
fn check_access_level(request: &HttpRequest<()>, min: AccessLevel) -> bool
{
    request.session()
        .get::<AccessLevel>(ACCESS_LEVEL_COOKIE)
        .unwrap_or_else(|err| {
            error!("Failed to read access level: {}", err);
            None
        })
        .map(|level| level >= min)
        .unwrap_or(false)
}

/// Creates an `HttpResponse` redirecting the user to the login page
///
/// After successful login, the user will be redirected to the page specified by `dest`. This
/// parameter is assumed to be the name of an Actix resource as set by the `Resource::name` method.
fn login_redirect(request: &HttpRequest<()>, dest: &str) -> HttpResponse
{
    let mut url = request.url_for_static(RES_LOGIN)
        .expect("Reverse-lookup of login resource failed");

    request.url_for_static(dest)
        .map(|dest| url.set_query(Some(&format!("dest={}", dest.path()))))
        .unwrap_or_else(|err|
            warn!("Reverse-lookup of destination resource \"{}\" failed: {}", dest, err)
        );

    HttpResponse::Found()
        .header(LOCATION, url.as_str())
        .finish()
}

//#endregion


//#region Resource Handlers

/// Returns the admin page's *GET* handler
fn get_admin(templates: Arc<Tera>) -> impl Fn(&HttpRequest<()>) -> HttpResponse
{
    move |request| {
        if check_access_level(request, AccessLevel::Administrative) {
            // TODO: create an admin template
            render(&templates, "admin.html")
        } else {
            login_redirect(request, RES_ADMIN)
        }
    }
}

/// Returns the home page's *GET* handler
fn get_home(templates: Arc<Tera>) -> impl Fn(&HttpRequest<()>) -> HttpResponse
{
    move |request| {
        if check_access_level(request, AccessLevel::Any) {
            render(&templates, "home.html")
        } else {
            login_redirect(request, RES_HOME)
        }
    }
}

/// Returns the login page's *GET* handler
fn get_login(templates: Arc<Tera>) -> impl Fn(&HttpRequest<()>) -> HttpResponse
{
    move |_| {
        render(&templates, "login.html")
    }
}

//#endregion


//#region Actix Application

// Unique resource names
const RES_ADMIN: &str = "admin";
const RES_HOME: &str = "home";
const RES_LOGIN: &str = "login";

/// Configures the admin resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_admin(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_ADMIN);
        resource.get().f(get_admin(templates));
    }
}

/// Configures the home resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_home(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_HOME);
        resource.get().f(get_home(templates));
    }
}

/// Configures the login resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_login(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_LOGIN);
        resource.get().f(get_login(templates));
        // TODO: on POST, validate creds and redirect to destination, or re-render login w/ error
    }
}

/// Returns an Actix application that provides LunaCam's user interface
pub fn app(secrets: Config<Secrets>, templates: Arc<Tera>, config: &SystemConfig) -> App
{
    trace!("initializing UI application");

    let mut app = App::new()
        .middleware(Logger::default())
        .middleware(SessionStorage::new(
            CookieSessionBackend::private(&secrets.read().session_key)
                .name("lc-session")
        ))
        .resource("/", res_home(templates.clone()))
        .resource("/login/", res_login(templates.clone()))
        .resource("/admin/", res_admin(templates.clone()));

    // Inability to serve static files is an error, but not necessarily fatal
    match StaticFiles::new(&config.static_path) {
        Ok(handler) => app = app.handler("/static", handler),
        Err(err) => error!("Failed to open static files handler: {}", err),
    }

    app
}

//#endregion
