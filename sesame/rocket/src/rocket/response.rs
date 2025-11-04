use crate::rocket::data::PConData;
use crate::rocket::request::PConRequest;
use crate::rocket::{PConRedirect, PConTemplate};
use rocket::http::ContentType;
use rocket::Either;
use sesame::context::{Context, ContextData};
use sesame::extensions::{ExtensionContext, SesameExtension};
use sesame::pcon::PCon;
use sesame::policy::{Policy, Reason};
use std::fs::File;
use std::result::Result;

// Our wrapper around response, disallows applications from looking at response
// in plain text.
pub struct PConResponse<'a> {
    response: rocket::response::Response<'a>,
}
impl<'a> PConResponse<'a> {
    pub fn new(response: rocket::response::Response<'a>) -> Self {
        PConResponse { response }
    }
    pub(crate) fn get_response(self) -> rocket::response::Response<'a> {
        self.response
    }
}

// The outcome of executing a handler / the outcome of an endpoint.
pub enum PConResponseOutcome<'a> {
    Success(PConResponse<'a>),
    Failure(rocket::http::Status),
    Forward(PConData<'a>),
}
impl<'a, 'r, 'o: 'r> PConResponseOutcome<'a> {
    pub fn from<R: PConResponder<'a, 'r, 'o>>(
        request: PConRequest<'a, 'r>,
        responder: R,
    ) -> PConResponseOutcome<'a> {
        match responder.respond_to(request) {
            Ok(response) => PConResponseOutcome::Success(response),
            Err(status) => PConResponseOutcome::Failure(status),
        }
    }
}

// A trait that signifies that implementors can be turned into a response.
pub type PConResponseResult<'a> = Result<PConResponse<'a>, rocket::http::Status>;

pub trait PConResponder<'a, 'r, 'o: 'r> {
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o>;
}

// Endpoints can return T: Responder, in case response is some non-protected data (e.g. a hardcoded
// string).
macro_rules! pcon_responder_impl {
    ($T: ty) => {
        impl<'a, 'r, 'o: 'r> PConResponder<'a, 'r, 'o> for $T {
            fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
                Ok(PConResponse::new(rocket::response::Responder::respond_to(
                    self,
                    request.get_request(),
                )?))
            }
        }
    };
}
pcon_responder_impl!(());
pcon_responder_impl!(String);
pcon_responder_impl!(&'o str);
pcon_responder_impl!(&'o [u8]);
pcon_responder_impl!(Vec<u8>);
pcon_responder_impl!(File);
pcon_responder_impl!(std::io::Error);

// Implement PConResponder for some containers.
impl<'a, 'r, 'o: 'r, T: PConResponder<'a, 'r, 'o>> PConResponder<'a, 'r, 'o> for (ContentType, T) {
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        let (content_type, responder) = self;
        let response = rocket::response::Response::build()
            .merge(responder.respond_to(request)?.response)
            .header(content_type)
            .ok();
        response.map(PConResponse::new)
    }
}
impl<'a, 'r, 'o: 'r, T: PConResponder<'a, 'r, 'o>> PConResponder<'a, 'r, 'o>
    for (rocket::http::Status, T)
{
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        let (status, responder) = self;
        let response = responder.respond_to(request)?.response;
        let response = rocket::response::Response::build_from(response)
            .status(status)
            .ok();
        response.map(PConResponse::new)
    }
}
impl<'a, 'r, 'o: 'r, T1: PConResponder<'a, 'r, 'o>, T2: PConResponder<'a, 'r, 'o>>
    PConResponder<'a, 'r, 'o> for Either<T1, T2>
{
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        match self {
            Either::Left(left) => left.respond_to(request),
            Either::Right(right) => right.respond_to(request),
        }
    }
}
impl<'a, 'r, 'o: 'r, T: PConResponder<'a, 'r, 'o>> PConResponder<'a, 'r, 'o> for Option<T> {
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        match self {
            Some(t) => t.respond_to(request),
            None => {
                let request = request.get_request();
                let response = rocket::response::Responder::respond_to(None::<()>, request)?;
                Ok(PConResponse::new(response))
            }
        }
    }
}
impl<'a, 'r, 'o: 'r, T: PConResponder<'a, 'r, 'o>, E: PConResponder<'a, 'r, 'o>>
    PConResponder<'a, 'r, 'o> for Result<T, E>
{
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        match self {
            Ok(t) => t.respond_to(request),
            Err(t) => t.respond_to(request),
        }
    }
}

// Endpoint functions can return this type in case they want to dynamically decide whether
// to render some page or redirect.
pub enum PConResponseEnum {
    Redirect(PConRedirect),
    Template(PConTemplate),
}
impl From<PConRedirect> for PConResponseEnum {
    fn from(value: PConRedirect) -> Self {
        PConResponseEnum::Redirect(value)
    }
}
impl From<PConTemplate> for PConResponseEnum {
    fn from(value: PConTemplate) -> Self {
        PConResponseEnum::Template(value)
    }
}
impl<'a, 'r> PConResponder<'a, 'r, 'static> for PConResponseEnum {
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'static> {
        match self {
            PConResponseEnum::Redirect(redirect) => redirect.respond_to(request),
            PConResponseEnum::Template(template) => template.respond_to(request),
        }
    }
}

// Endpoints can return (PCon<T: Responder, Policy>, Context) which Sesame eventually turns into
// T after a policy check.
pub struct ContextResponse<T, P: Policy, D: ContextData>(pub PCon<T, P>, pub Context<D>);
impl<T, P: Policy, D: ContextData> From<(PCon<T, P>, Context<D>)> for ContextResponse<T, P, D> {
    fn from((pcon, context): (PCon<T, P>, Context<D>)) -> Self {
        Self(pcon, context)
    }
}
impl<'a, 'r, 'o: 'r, T: PConResponder<'a, 'r, 'o>, P: Policy, D: ContextData>
    PConResponder<'a, 'r, 'o> for ContextResponse<T, P, D>
{
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'o> {
        let mut extension = ResponsePolicyChecker::new(request);
        let (pcon, context) = (self.0, self.1);
        let context = ExtensionContext::new(context);
        match pcon.checked_extension(&mut extension, &context, Reason::Response) {
            Ok(response) => response,
            Err(err) => err.respond_to(request),
        }
    }
}

// Sesame extension that performs policy check then renders template if successful.
struct ResponsePolicyChecker<'a, 'r> {
    request: PConRequest<'a, 'r>,
}
impl<'a, 'r> ResponsePolicyChecker<'a, 'r> {
    fn new(request: PConRequest<'a, 'r>) -> Self {
        ResponsePolicyChecker { request }
    }
}
impl<'a, 'r, 'o: 'r, T: PConResponder<'a, 'r, 'o>, P: Policy>
    SesameExtension<T, P, PConResponseResult<'o>> for ResponsePolicyChecker<'a, 'r>
{
    fn apply(&mut self, data: T, _policy: P) -> PConResponseResult<'o> {
        data.respond_to(self.request)
    }
}
