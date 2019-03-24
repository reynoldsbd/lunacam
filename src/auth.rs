use std::sync::Arc;

use actix_web::{Form, HttpRequest, HttpResponse, Request};
use actix_web::dev::Resource;
use actix_web::http::header;
use actix_web::middleware::{Middleware, Started};
use actix_web::middleware::session::RequestSession;
use actix_web::pred;
use actix_web::pred::Predicate;

use serde::{Deserialize, Serialize};


/// Matches only the specified path
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


/// Login form content
#[derive(Deserialize)]
pub struct LoginForm {
    password: String,
}


/// Session data key for user type
const USER_TYPE_COOKIE_NAME: &str = "usertype";


/// Defines access type for a signed-in user
#[derive(Debug, Deserialize, PartialEq, Serialize)]
enum UserType {

    /// User has regular level of access
    Regular,

    /// User has administrative level of access
    Administrator,
}


/// Prevents unauthorized access to restricted resources
struct AuthorizationMiddleware;

impl<S> Middleware<S> for AuthorizationMiddleware {
    fn start(&self, request: &HttpRequest<S>) -> Result<Started, actix_web::Error> {
        let user_type = request.session().get(USER_TYPE_COOKIE_NAME)?;
        let path = request.path();

        // If user does not have sufficient access, redirect to the login page
        // TODO: find a better way to map page => access level
        if (path == "/" && user_type == None) ||
            (path == "/admin" && user_type != Some(UserType::Administrator)) {
            Ok(Started::Response(
                HttpResponse::Found()
                    .header(header::LOCATION, "/login")
                    .finish()
            ))
        }

        else {
            Ok(Started::Done)
        }
    }
}


/// Manages user authentication
pub struct Authenticator {
    user_pw: String,
    admin_pw: String,
}

impl Authenticator {

    /// Returns UserType for the given password (or None if password is invalid)
    fn authenticate(&self, pw: &str) -> Option<UserType> {
        if pw == self.user_pw {
            Some(UserType::Regular)
        } else if pw == self.admin_pw {
            Some(UserType::Administrator)
        } else {
            None
        }
    }

    /// Handles POST submission of the login page
    ///
    /// If specified password is valid, user's access level is persisted in a secure cookie and user
    /// is redirected to home page. Otherwise, user is sent back to the login page to try again.
    fn handle(
        &self,
        req: HttpRequest,
        form: Form<LoginForm>
    ) -> Result<HttpResponse, actix_web::Error> {
        if let Some(user_type) = self.authenticate(&form.password) {
            req.session().set(USER_TYPE_COOKIE_NAME, user_type)?;
            Ok(
                HttpResponse::Found()
                    // TODO: use query param to specify redirect location
                    .header(header::LOCATION, "/")
                    .finish()
            )
        }
        else {
            Ok(
                // TODO: instead of redirect, directly render login page with error message
                HttpResponse::Found()
                    .header(header::LOCATION, "/login")
                    .finish()
            )
        }
    }

    /// Initializes authentication for the given Resource
    pub fn register(resource: &mut Resource, user_pw: String, admin_pw: String) {
        let authenticator = Arc::new(Authenticator {
            user_pw: user_pw,
            admin_pw: admin_pw,
        });

        resource.middleware(AuthorizationMiddleware { });
        resource.route()
            .filter(pred::Post())
            .filter(PathPredicate::new("/login"))
            .with(move |req, form| authenticator.handle(req, form));
    }
}
