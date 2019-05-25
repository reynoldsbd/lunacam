//! Serves the LunaCam user interface


// TODO: error handling


//#region Usings

use std::sync::Arc;

use actix_web::{Form, HttpRequest, HttpResponse, Scope};
use actix_web::dev::Resource;
use actix_web::http::header::LOCATION;

use log::{debug, error, trace, warn};

use serde::{Deserialize};

use tera::{Context, Tera};

use crate::sec;
use crate::sec::{AccessLevel, Authenticator};

//#endregion


//#region Helpers

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

/// Creates an `HttpResponse` redirecting the user to the login page
///
/// Sets the `dest` query parameter to the current request's path, allowing `post_login` to
/// redirect back after a successful login.
fn login_redirect(request: &HttpRequest<()>) -> HttpResponse
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


//#region Resource Handlers

/// Returns the admin page's GET handler
fn get_admin(templates: Arc<Tera>) -> impl Fn(&HttpRequest<()>) -> HttpResponse
{
    move |_| {
        render(&templates, "admin.html")
    }
}

/// Returns the home page's GET handler
fn get_home(templates: Arc<Tera>) -> impl Fn(&HttpRequest<()>) -> HttpResponse
{
    move |_| {
        render(&templates, "home.html")
    }
}

/// Returns the login page's GET handler
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

/// Returns the login page's POST handler
fn post_login(auth: Authenticator, templates: Arc<Tera>) -> impl Fn(HttpRequest<()>, Form<LoginForm>) -> HttpResponse
{
    // TODO: won't need a handle to sec::Authenticator if we make a static sec::authenticate fn

    move |request, form| {
        if auth.authenticate(&request, &form.password) {

            // TODO: this is hideous
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
        resource.middleware(sec::require(AccessLevel::Administrator, login_redirect));
        resource.get().f(get_admin(templates));
    }
}

/// Configures the home resource
fn res_home(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_HOME);
        resource.middleware(sec::require(AccessLevel::User, login_redirect));
        resource.get().f(get_home(templates));
    }
}

/// Configures the login resource
fn res_login(auth: Authenticator, templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.name(RES_LOGIN);
        resource.get().f(get_login(templates.clone()));
        resource.post().with(post_login(auth, templates));
    }
}

/// Configures LunaCam's UI scope
pub fn scope(auth: Authenticator, templates: Arc<Tera>) -> impl FnOnce(Scope<()>) -> Scope<()>
{
    move |scope| {
        trace!("configuring UI scope");

        scope
            .resource("/", res_home(templates.clone()))
            .resource("/login/", res_login(auth.clone(), templates.clone()))
            .resource("/admin/", res_admin(templates.clone()))
    }
}

//#endregion
