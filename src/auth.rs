use actix_web::{HttpRequest, HttpResponse, Request};
use actix_web::http::header;
use actix_web::pred::Predicate;


/// Path-based predicate for the login handler
pub struct LoginPredicate;

impl<S> Predicate<S> for LoginPredicate {
    fn check(&self, request: &Request, _: &S) -> bool {
        request.path() == "/login"
    }
}


/// Handles submission of login page
pub fn post_login_handler(request: &HttpRequest) -> HttpResponse {
    // TODO: extract password
    dbg!("LOGIN WAS POSTED!!");

    HttpResponse::Found()
        .header(header::LOCATION, "/")
        .finish()
}
