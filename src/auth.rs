//! Authentication system

// TODO: get rid of old stuff
// TODO: rename "Authenticator" to something more appropriate (maybe "SecurityContext")

#![allow(dead_code)]
#![allow(unused_imports)]


//#region Usings

use std::result;
use std::sync::Arc;

use actix::{Actor, Addr, Context, Handler, MailboxError, Message};

use actix_web::{Form, HttpRequest, HttpResponse, Request};
use actix_web::dev::Resource;
use actix_web::http::header;
use actix_web::middleware::{Middleware, Started};
use actix_web::middleware::session::RequestSession;
use actix_web::pred;
use actix_web::pred::Predicate;

use derive_more::Display;

use futures::future::Future;

use log::{error};

use serde::{Deserialize, Serialize};

use crate::config;
use crate::config::{Config, SystemConfig};

//#endregion


//#region Old

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
pub enum UserType {

    /// User has regular level of access
    Regular,

    /// User has administrative level of access
    Administrator,
}


/// Prevents unauthorized access to restricted resources
struct AuthorizationMiddleware;

impl<S> Middleware<S> for AuthorizationMiddleware {
    fn start(&self, request: &HttpRequest<S>) -> result::Result<Started, actix_web::Error> {
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
    ) -> result::Result<HttpResponse, actix_web::Error> {
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

//#endregion


//#region Error Handling

#[derive(Debug, Display)]
pub enum Error {
    AuthFailed,
    Config(config::Error),
    Mailbox(MailboxError),
}

impl From<config::Error> for Error {
    fn from(err: config::Error) -> Self {
        Error::Config(err)
    }
}

impl From<MailboxError> for Error {
    fn from(err: MailboxError) -> Self {
        Error::Mailbox(err)
    }
}

pub type Result<T> = result::Result<T, Error>;

//#endregion


//#region Authenticator

// TODO: manually impl Default for AuthConfig to automatically randomize secret
// TODO: (or, make Secret a newtype and impl Default for only it)

pub type Secret = [u8; 32];

#[derive(Clone, Default, Deserialize, Serialize)]
struct AuthConfig {
    admin_pw: String,
    secret: Secret,
    user_pw: String,
}

// impl UserConfig for AuthConfig {}

// pub struct NewAuthenticator {
//     config: Addr<Config<AuthConfig>>,
// }

// impl NewAuthenticator {

//     pub fn new(config: &SystemConfig) -> Result<Self> {
//         Ok(NewAuthenticator {
//             config: Config::new("auth", config)?
//                 .start()
//         })
//     }
// }

// impl Actor for NewAuthenticator {
//     type Context = Context<Self>;
// }

pub struct Authenticate(String);

impl Message for Authenticate {
    type Result = Result<UserType>;
}

// impl Handler<Authenticate> for NewAuthenticator {
//     type Result = Box<dyn Future<Item = UserType, Error = Error>>;

//     fn handle(&mut self, msg: Authenticate, _: &mut Context<Self>) -> Self::Result {
//         let Authenticate(pw) = msg;
//         let fut = self.config.send(config::LoadConfig::new())
//             .map_err(|err| {
//                 error!("unexpected mailbox error ({})", err);
//                 Error::from(err)
//             })
//             .and_then(|res| res.map_err(|err| Error::from(err)))
//             .and_then(move |cfg| {
//                 if pw == cfg.admin_pw {
//                     Ok(UserType::Administrator)
//                 } else if pw == cfg.user_pw {
//                     Ok(UserType::Regular)
//                 } else {
//                     Err(Error::AuthFailed)
//                 }
//             });
//         Box::new(fut)
//     }
// }

pub struct GetSecret;

impl Message for GetSecret {
    type Result = Result<Secret>;
}

// impl Handler<GetSecret> for NewAuthenticator {
//     type Result = Box<dyn Future<Item = Secret, Error = Error>>;

//     fn handle(&mut self, msg: GetSecret, _: &mut Context<Self>) -> Self::Result {
//         let fut = self.config.send(config::LoadConfig::new())
//             .map_err(|err| {
//                 error!("unexpected mailbox error ({})", err);
//                 Error::from(err)
//             })
//             .and_then(|res| res.map_err(|err| Error::from(err)))
//             .map(|cfg| cfg.secret.clone());
//         Box::new(fut)
//     }
// }

//#endregion
