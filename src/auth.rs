use std::sync::Arc;

use actix_web::{AsyncResponder, Form, FromRequest, HttpRequest, HttpResponse, Request};
use actix_web::dev::{Handler, Resource};
use actix_web::http::header;
use actix_web::middleware::session::RequestSession;
use actix_web::pred;
use actix_web::pred::Predicate;

use futures::Future;

use serde::{Deserialize, Serialize};


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


const USER_TYPE_COOKIE_NAME: &str = "usertype";


#[derive(Debug, Deserialize, Serialize)]
enum UserType {
    Regular,
    Administrator,
}


pub struct Authenticator {
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

    fn handle(
        &self,
        req: HttpRequest,
        form: Form<LoginForm>
    ) -> Result<HttpResponse, actix_web::Error> {
        if let Some(user_type) = self.authenticate(&form.password) {
            req.session().set(USER_TYPE_COOKIE_NAME, user_type)?;
            Ok(
                HttpResponse::Found()
                    .header(header::LOCATION, "/")
                    .finish()
            )
        }
        else {
            Ok(
                HttpResponse::Found()
                    .header(header::LOCATION, "/login")
                    .finish()
            )
        }
    }

    pub fn register(resource: &mut Resource, user_pw: String, admin_pw: String) {
        let authenticator = Arc::new(Authenticator {
            user_pw: user_pw,
            admin_pw: admin_pw,
        });

        resource.route()
            .filter(pred::Post())
            .filter(PathPredicate::new("/login"))
            .with(move |req, form| authenticator.handle(req, form));
    }
}
