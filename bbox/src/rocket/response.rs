use dynfmt::{Format, SimpleCurlyFormat};
use std::result::Result;

use crate::bbox::BBoxRender;
use crate::rocket::data::BBoxData;
use crate::rocket::request::BBoxRequest;

// Our wrapper around response, disallows applications from looking at response
// in plain text.
pub struct BBoxResponse<'r> {
    response: rocket::response::Response<'r>,
}
impl<'r> BBoxResponse<'r> {
    pub fn new(response: rocket::response::Response<'r>) -> Self {
        BBoxResponse { response }
    }
    pub(crate) fn get_response(self) -> rocket::response::Response<'r> {
        self.response
    }
}

// The outcome of executing a handler / the outcome of an endpoint.
pub enum BBoxResponseOutcome<'r> {
    Success(BBoxResponse<'r>),
    Failure(rocket::http::Status),
    Forward(BBoxData<'r>),
}
impl<'r, 'o: 'r> BBoxResponseOutcome<'o> {
    pub fn from<R: BBoxResponder<'r, 'o>>(
        request: &BBoxRequest<'r, '_>,
        responder: R,
    ) -> BBoxResponseOutcome<'r> {
        match responder.respond_to(request) {
            Result::Ok(response) => BBoxResponseOutcome::Success(response),
            Result::Err(status) => BBoxResponseOutcome::Failure(status),
        }
    }
}

// A trait that signifies that implementors can be turned into a response.
pub type BBoxResponseResult<'r> = Result<BBoxResponse<'r>, rocket::http::Status>;
pub trait BBoxResponder<'r, 'o: 'r> {
    fn respond_to(self, request: &BBoxRequest<'r, '_>) -> BBoxResponseResult<'o>;
}

// A redirect response.
pub struct BBoxRedirect {
    redirect: rocket::response::Redirect,
}
impl BBoxRedirect {
    pub fn redirect(name: &str, params: Vec<&dyn BBoxRender>) -> Self {
        let params = params
            .iter()
            .map(|x| x.render().try_unbox().unwrap().clone())
            .collect::<Vec<_>>();
        let formatted_str = SimpleCurlyFormat.format(name, params).unwrap();
        BBoxRedirect {
            redirect: rocket::response::Redirect::to(Into::<String>::into(formatted_str)),
        }
    }
}
impl<'r, 'o: 'r> BBoxResponder<'r, 'o> for BBoxRedirect {
    fn respond_to(self, request: &BBoxRequest<'r, '_>) -> BBoxResponseResult<'o> {
        use rocket::response::Responder;
        match self.redirect.respond_to(request.get_request()) {
            Result::Ok(response) => Result::Ok(BBoxResponse::new(response)),
            Result::Err(e) => Result::Err(e),
        }
    }
}
