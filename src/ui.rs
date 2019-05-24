//! Serves the LunaCam user interface


// TODO: error handling


//#region Usings

use std::sync::Arc;

use actix_web::{Form, HttpRequest, HttpResponse, Scope};
use actix_web::dev::Resource;
use actix_web::http::header::LOCATION;
use actix_web::middleware::session::{RequestSession};

use log::{debug, error, trace, warn};

use rand::Rng;

use serde::{Deserialize, Serialize};

use tera::{Context, Tera};

use crate::config::{Config};

//#endregion


//#region Secrets

// TODO: wouldn't need to be public if Config were singleton or Actix service

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

// TODO: use an actor to support reloading

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


//#region Authentication and Login

// TODO: some of this needs to be usable from the api module

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

#[derive(Deserialize)]
struct LoginForm
{
    password: String,
}

fn post_login(secrets: Config<Secrets>, templates: Arc<Tera>) -> impl Fn(HttpRequest<()>, Form<LoginForm>) -> HttpResponse
{
    let authenticate = move |password: &str| {
        let secrets = secrets.read();
        if password == &secrets.user_pw {
            Some(AccessLevel::Any)
        } else if password == &secrets.admin_pw {
            Some(AccessLevel::Administrative)
        } else {
            debug!("authentication failed");
            None
        }
    };

    move |request, form| {
        if let Some(level) = authenticate(&form.password) {
            request.session().set(ACCESS_LEVEL_COOKIE, level)
                .unwrap_or_else(|err| error!("Failed to set access level cookie: {}", err));
            HttpResponse::Found()
                .header(
                    LOCATION,
                    request.query()
                        .get("dest")
                        .map(|dest| dest.to_owned())
                        .unwrap_or_else(||
                            request.url_for_static(RES_HOME)
                                .map(|url| url.as_str().to_owned())
                                .unwrap_or_else(|err| {
                                    error!("Reverse-lookup of home resource failed: {}", err);
                                    "/".to_owned()
                                })
                        )
                )
                .finish()

        } else {
            warn!("failed authentication attempt");
            // TODO: display user-visible warning
            render(&templates, "login.html")
        }
    }
}

//#endregion


//#region Actix Application

// Unique resource names
const RES_ADMIN: &str = "admin";
const RES_HOME: &str = "home";
const RES_LOGIN: &str = "login";

/// Configures the admin resource
fn res_admin(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_ADMIN);
        resource.get().f(get_admin(templates));
    }
}

/// Configures the home resource
fn res_home(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_HOME);
        resource.get().f(get_home(templates));
    }
}

/// Configures the login resource
fn res_login(secrets: Config<Secrets>, templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_LOGIN);
        resource.get().f(get_login(templates.clone()));
        resource.post().with(post_login(secrets, templates));
    }
}

/// Configures LunaCam's UI scope
pub fn scope(secrets: Config<Secrets>, templates: Arc<Tera>) -> impl FnOnce(Scope<()>) -> Scope<()>
{
    move |scope| {
        trace!("configuring UI scope");

        scope
            .resource("/", res_home(templates.clone()))
            .resource("/login/", res_login(secrets.clone(), templates.clone()))
            .resource("/admin/", res_admin(templates.clone()))
    }
}

//#endregion
