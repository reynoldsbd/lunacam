//! Error handling used throughout LunaCam

use actix_web::HttpResponse;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use diesel::result::Error as DieselError;
use serde::Serialize;


/// Error type generated by LunaCam
#[derive(Debug, Display)]
pub enum Error {

    /// Error produced by a failed web request
    #[display(fmt = "{}", _1)]
    Web(StatusCode, &'static str),

    /// Error propagated from a third-party library
    External(Box<dyn std::error::Error>),
}

impl Error {

    /// Creates a new `Error` with custom HTTP status code and message
    pub fn web<T>(status: StatusCode, msg: &'static str) -> Result<T> {
        Err(Self::Web(status, msg))
    }
}

impl<T: std::error::Error + 'static> From<T> for Error {
    fn from(err: T) -> Self {
        Self::External(Box::new(err))
    }
}

impl ResponseError for Error {

    fn error_response(&self) -> HttpResponse {

        #[derive(Serialize)]
        struct ErrorBody<'a> {
            message: &'a str
        }

        let status = match self {
            Self::Web(status, _) => *status,
            Self::External(err) if err.downcast_ref() == Some(&DieselError::NotFound) =>
                StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ErrorBody {
            message: &format!("{}", self),
        };

        HttpResponse::build(status)
            .json(body)
    }

    fn render_response(&self) -> HttpResponse {
        self.error_response()
    }
}


/// Result type generated by LunaCam
pub type Result<T> = std::result::Result<T, Error>;
