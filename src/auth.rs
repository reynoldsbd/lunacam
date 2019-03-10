use actix_web::{Form, FromRequest, HttpRequest, HttpResponse, Request};
use actix_web::dev::{Handler, Resource};
use actix_web::http::header;
use actix_web::pred;
use actix_web::pred::Predicate;

use futures::Future;

use serde::Deserialize;


/// Path-based route predicate
pub struct PathPredicate(String);

impl PathPredicate {

    /// Creates a new PathPredicate
    pub fn new<S>(s: S) -> PathPredicate
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
pub struct LoginHandler {
    user_pw: String,
    admin_pw: String,
}

impl LoginHandler {

    pub fn register(resource: &mut Resource, user_pw: String, admin_pw: String) {
        resource.route()
            .filter(pred::Post())
            .filter(PathPredicate::new("/login"))
            .h(LoginHandler {
                user_pw: user_pw,
                admin_pw: admin_pw,
            });
    }
}

impl Handler<()> for LoginHandler {
    type Result = Box<dyn Future<Item=HttpResponse, Error=actix_web::Error>>;

    fn handle(&self, request: &HttpRequest<()>) -> Self::Result {
        // Don't fully understand why, but the resturn value must be 'static, which means we can't
        // reference self from inside any callbacks.
        let user_pw = self.user_pw.clone();
        let admin_pw = self.admin_pw.clone();

        Box::new(
            Form::<LoginForm>::extract(request)
                .map(move |f| {
                    if f.password == user_pw {
                        // TODO: set cookie user
                        println!("Authentication successful for a normal user");
                    } else if f.password == admin_pw {
                        // TODO: set cookie to admin
                        println!("Authentication successful for an admin");
                    } else {
                        // TODO: render login page directly, instead of redirecting
                        // TODO: show an error message
                        return HttpResponse::Found()
                            .header(header::LOCATION, "/login")
                            .finish();
                    }

                    HttpResponse::Found()
                        .header(header::LOCATION, "/")
                        .finish()
                })
        )
    }
}
