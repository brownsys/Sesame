use std::result::Result;
use crate::rocket::{BBoxRedirect, BBoxTemplate};

use crate::rocket::data::BBoxData;
use crate::rocket::request::BBoxRequest;

// Our wrapper around response, disallows applications from looking at response
// in plain text.
pub struct BBoxResponse<'a> {
    response: rocket::response::Response<'a>,
}
impl<'a> BBoxResponse<'a> {
    pub fn new(response: rocket::response::Response<'a>) -> Self {
        BBoxResponse { response }
    }
    pub(crate) fn get_response(self) -> rocket::response::Response<'a> {
        self.response
    }
}

// The outcome of executing a handler / the outcome of an endpoint.
pub enum BBoxResponseOutcome<'a> {
    Success(BBoxResponse<'a>),
    Failure(rocket::http::Status),
    Forward(BBoxData<'a>),
}
impl<'a, 'r> BBoxResponseOutcome<'a> {
    pub fn from<R: BBoxResponder<'a, 'r>>(
        request: BBoxRequest<'a, 'r>,
        responder: R,
    ) -> BBoxResponseOutcome<'a> {
        match responder.respond_to(request) {
            Result::Ok(response) => BBoxResponseOutcome::Success(response),
            Result::Err(status) => BBoxResponseOutcome::Failure(status),
        }
    }
}

// A trait that signifies that implementors can be turned into a response.
pub type BBoxResponseResult<'a> = Result<BBoxResponse<'a>, rocket::http::Status>;
pub trait BBoxResponder<'a, 'r> {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'a>;
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
impl<'a, 'r> BBoxResponder<'a, 'r> for BBoxResponseEnum {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'a> {
        match self {
            BBoxResponseEnum::Redirect(redirect) => redirect.respond_to(request),
            BBoxResponseEnum::Template(template) => template.respond_to(request),
        }
    }
}