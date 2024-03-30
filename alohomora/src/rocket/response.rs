use std::result::Result;
use crate::bbox::BBox;
use crate::context::{Context, ContextData, UnprotectedContext};
use crate::policy::{Policy, Reason};
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
impl<'a, 'r, 'o: 'r> BBoxResponseOutcome<'a> {
    pub fn from<R: BBoxResponder<'a, 'r, 'o>>(
        request: BBoxRequest<'a, 'r>,
        responder: R,
    ) -> BBoxResponseOutcome<'a> {
        match responder.respond_to(request) {
            Ok(response) => BBoxResponseOutcome::Success(response),
            Err(status) => BBoxResponseOutcome::Failure(status),
        }
    }
}

// A trait that signifies that implementors can be turned into a response.
pub type BBoxResponseResult<'a> = Result<BBoxResponse<'a>, rocket::http::Status>;
pub trait BBoxResponder<'a, 'r, 'o: 'a> {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'o>;
}

// Endpoints can return T: Responder, in case response is some non-protected data (e.g. a hardcoded
// string).
impl<'a, 'r, 'o: 'a, T: rocket::response::Responder<'a, 'o>> BBoxResponder<'a, 'r, 'o> for T {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'o> {
        Ok(BBoxResponse::new(self.respond_to(request.get_request())?))
    }
}

// Endpoints can return (BBox<T: Responder, Policy>, Context) which Alohomora eventually turns into
// T after a policy check.
pub struct ContextResponse<T, P: Policy, D: ContextData>(pub BBox<T, P>, pub Context<D>);
impl<T, P: Policy, D: ContextData> From<(BBox<T, P>, Context<D>)> for ContextResponse<T, P, D> {
    fn from((bbox, context): (BBox<T, P>, Context<D>)) -> Self {
        Self(bbox, context)
    }
}

impl<'a, 'r, 'o: 'a, T: rocket::response::Responder<'a, 'o>, P: Policy, D: ContextData> BBoxResponder<'a, 'r, 'o> for ContextResponse<T, P, D> {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'o> {
        let (bbox, context) = (self.0, self.1);
        let (t, p) = bbox.consume();
        let context = UnprotectedContext::from(context);
        if p.check(&context, Reason::Response) {
            Ok(BBoxResponse::new(t.respond_to(request.get_request())?))
        } else {
            Err(rocket::http::Status {
                code: 555,
            })
        }
    }
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
impl<'a, 'r> BBoxResponder<'a, 'r, 'static> for BBoxResponseEnum {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'static> {
        match self {
            BBoxResponseEnum::Redirect(redirect) => redirect.respond_to(request),
            BBoxResponseEnum::Template(template) => template.respond_to(request),
        }
    }
}