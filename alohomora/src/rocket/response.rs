use std::result::Result;
use crate::rocket::{BBoxRedirect, BBoxTemplate};

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

// Endpoint functions can return this type in case they want to dynamically decide whether
// to render some page or redirect.
pub enum BBoxResponseEnum {
    Redirect(BBoxRedirect),
    Template(BBoxTemplate),
}
impl From<BBoxRedirect> for BBoxResponseEnum {
    fn from(value: BBoxRedirect) -> Self {
        BBoxResponseEnum::Redirect(value)
    }
}
impl From<BBoxTemplate> for BBoxResponseEnum {
    fn from(value: BBoxTemplate) -> Self {
        BBoxResponseEnum::Template(value)
    }
}
impl<'r, 'o: 'r> BBoxResponder<'r, 'o> for BBoxResponseEnum {
    fn respond_to(self, request: &BBoxRequest<'r, '_>) -> BBoxResponseResult<'o> {
        match self {
            BBoxResponseEnum::Redirect(redirect) => redirect.respond_to(request),
            BBoxResponseEnum::Template(template) => template.respond_to(request),
        }
    }
}