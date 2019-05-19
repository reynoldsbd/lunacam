//! Serves the LunaCam user interface


//#region Usings

use std::sync::Arc;

use actix_web::{App, HttpResponse};
use actix_web::dev::Resource;
use actix_web::fs::StaticFiles;
use actix_web::http::header::LOCATION;
use actix_web::middleware::Logger;
use actix_web::middleware::session::{CookieSessionBackend, RequestSession, SessionStorage};

use log::{debug, error, info, trace};

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

/// Defines access level for a signed-in user
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum UserType
{
    /// User has regular level of access
    Regular,

    /// User has administrative level of access
    Administrator,
}

/// Name of secure cookie used to store user type
const USER_TYPE_COOKIE: &str = "userType";

//#endregion


//#region Actix Application

// TODO: figure out how to factor out common implementation of identity verification

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

/// Configures the */index/* resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_index(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.get().f(move |request| {
            trace!("handling GET request for index page");
            let user_type: Option<UserType> = request.session()
                .get(USER_TYPE_COOKIE)
                .unwrap_or_else(|err| {
                    error!("Failed to access user type cookie: {}", err);
                    None
                });

            // TODO: uncomment once login page is ready
            if user_type.is_some() {
                info!("user type is {:?}", user_type);
                // render(&templates, "index.html")
            } else {
                info!("user is not logged in");
                // HttpResponse::Found()
                //     .header(LOCATION, "/login")
                //     .finish()
            }
            render(&templates, "index.html")
        });
    }
}

/// Configures the */login/* resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_login(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.get().f(move |_| {
            trace!("handling GET request for login page");
            render(&templates, "login.html")
        });
        // TODO: on POST, validate creds and redirect to destination, or re-render login w/ error
    }
}

/// Configures the */admin/* resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_admin(templates: Arc<Tera>) -> impl FnOnce(&mut Resource<()>)
{
    |resource| {
        resource.get().f(move |_| {
            trace!("handling GET request for admin page");
            // TODO: check admin status or redirect to login
            // TODO: create an admin template
            render(&templates, "admin.html")
        });
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
        .resource("/", res_index(templates.clone()))
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
