use std::sync::Arc;

use actix_web::{AsyncResponder, Form, FromRequest, HttpRequest, HttpResponse, Request};
use actix_web::dev::{Handler, Resource};
use actix_web::http::header;
use actix_web::pred;
use actix_web::pred::Predicate;

use futures::Future;

use serde::Deserialize;


#[derive(Debug)]
enum UserType {
    Regular,
    Administrator,
}


struct Authenticator {
    user_pw: String,
    admin_pw: String,
}

impl Authenticator {

    fn authenticate(&self, pw: &str) -> Option<UserType> {
        if pw == self.user_pw {
            Some(UserType::Regular)
        } else if pw == self.admin_pw {
            Some(UserType::Administrator)
        } else {
            None
        }
    }
}


/// Path-based route predicate
struct PathPredicate(String);

impl PathPredicate {

    /// Creates a new PathPredicate
    fn new<S>(s: S) -> PathPredicate
    where S: AsRef<str> {
        PathPredicate(s.as_ref().to_owned())
    }
}

impl<S> Predicate<S> for PathPredicate {
    fn check(&self, request: &Request, _: &S) -> bool {
        request.path() == self.0
    }
}


/// Defines expected login form content
#[derive(Deserialize)]
pub struct LoginForm {
    password: String,
}


/// Handles POST of the login page
#[derive(Clone)]
pub struct LoginHandler(Arc<Authenticator>);

impl LoginHandler {

    pub fn register(resource: &mut Resource, user_pw: String, admin_pw: String) {
        resource.route()
            .filter(pred::Post())
            .filter(PathPredicate::new("/login"))
            .h(LoginHandler(Arc::new(Authenticator {
                user_pw: user_pw,
                admin_pw: admin_pw,
            })));
    }
}

impl Handler<()> for LoginHandler {
    type Result = Box<dyn Future<Item=HttpResponse, Error=actix_web::Error>>;

    fn handle(&self, request: &HttpRequest<()>) -> Self::Result {
        // Don't fully understand why, but the resturn value must be 'static, which means we can't
        // reference self from inside any callbacks.
        let authenticator = self.0.clone();

        Form::<LoginForm>::extract(request)
            .map(move |f| match dbg!(authenticator.authenticate(&f.password)) {
                Some(_) => {
                    // TODO: set user type cookie
                    // TODO: redirect based on query param
                    HttpResponse::Found()
                        .header(header::LOCATION, "/")
                        .finish()
                },
                None => {
                    // TODO: directly render login page
                    // TODO: display some error message
                    HttpResponse::Found()
                        .header(header::LOCATION, "/login")
                        .finish()
                }
            })
            .responder()
    }
}
