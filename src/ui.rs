//! Serves the LunaCam user interface


//#region Usings

use std::rc::Rc;

use actix::{Addr};

use actix_web::{App, HttpResponse};
use actix_web::dev::Resource;
use actix_web::fs::StaticFiles;
use actix_web::middleware::Logger;
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};

use futures::future::Future;

use log::{error};

use tera::{compile_templates, Context, Tera};

use crate::auth::{NewAuthenticator, GetSecret};
use crate::config::SystemConfig;

//#endregion


//#region Actix Application

/// Renders the specified template to an `HttpResponse`
///
/// If an error occurs while rendering the template, the error error message is logged and written
/// to the returned response.
fn render(tera: &Tera, name: &str) -> HttpResponse {
    tera.render(name, &Context::new())
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
fn res_index(tera: Rc<Tera>) -> impl FnOnce(&mut Resource<()>) {
    |resource| {
        // TODO: check login status or redirect to login
        resource.get().f(move |_| render(&tera, "index.html"));
    }
}

/// Configures the */login/* resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_login(tera: Rc<Tera>) -> impl FnOnce(&mut Resource<()>) {
    |resource| {
        resource.get().f(move |_| render(&tera, "login.html"));
        // TODO: on POST, validate creds and redirect to destination, or re-render login w/ error
    }
}

/// Configures the */admin/* resource
///
/// This function returns a callback that can be passed to the `App::resource` method
fn res_admin(tera: Rc<Tera>) -> impl FnOnce(&mut Resource<()>) {
    |resource| {
        // TODO: check admin status or redirect to login
        // TODO: create an admin template
        resource.get().f(move |_| render(&tera, "admin.html"));
    }
}

/// Returns an Actix application that provides LunaCam's user interface
pub fn app(auth: Addr<NewAuthenticator>, config: &SystemConfig) -> App {

    // TODO: shouldn't need to pass 2 messages for this
    let secret = auth.send(GetSecret)
        .wait()
        .expect("failed to communicate with authenticator")
        .expect("authenticator failed to retrieve secret");
    let tera = Rc::new(compile_templates!(&format!("{}/**/*", &config.template_path)));

    let mut app = App::new()
        .middleware(Logger::default())
        .middleware(SessionStorage::new(
            CookieSessionBackend::private(&secret)
                .name("lc-session")
        ))
        .resource("/", res_index(tera.clone()))
        .resource("/login/", res_login(tera.clone()))
        .resource("/admin/", res_admin(tera.clone()));

    // Inability to serve static files is an error, but not necessarily fatal
    match StaticFiles::new(&config.static_path) {
        Ok(handler) => app = app.handler("/static", handler),
        Err(err) => error!("Failed to open static files handler: {}", err),
    }

    app
}

//#endregion
