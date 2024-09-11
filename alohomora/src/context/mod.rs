use std::any::Any;
use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxData, FromBBoxRequest, FromBBoxRequestAndData};
use rocket::http::Status;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use crate::AlohomoraType;
use crate::fold::fold;

// Context Data must satisfy these requirements.
pub trait ContextData : AlohomoraType + Send + 'static {}
impl<D: AlohomoraType + Send + 'static> ContextData for D {}

// Context is generic over some developer defined data.
#[derive(Debug, Clone)]
pub struct Context<D: ContextData> {
    route: String,
    data: Option<D>,
}
impl<D: ContextData> Context<D> {
    pub fn route(&self) -> &str {
        &self.route
    }

    // Context cannot be manufactured.
    pub fn new(route: String, data: D) -> Self {
        Self {
            route,
            data: Some(data),
        }
    }

    // Can manufacture an empty context for ease of use when dealing with Alohomora APIs without boxes.
    pub fn empty() -> Self {
        Self {
            route: String::from(""),
            data: None,
        }
    }

    // Only for testing.
    pub fn data(&self) -> Option<&D> {
       self.data.as_ref()
    }
}

// The only way to construct a Context is to get via from a BBoxRequest using below trait.
// This also implies that D has to be constructed that way as well, meaning that any sensitive
// information stored in D (e.g. something from a cookie, like a UserID), will have to originate
// from the BBoxRequest (and thus be in BBox form, at least initially).
#[derive(Debug)]
pub enum ContextError {
    Unconstructible,
}

// Context can be constructed from just the request, or the request and the form data, depending on
// what the developer chooses for D.
#[rocket::async_trait]
impl<'a, 'r, D: ContextData + FromBBoxRequest<'a, 'r>> FromBBoxRequest<'a, 'r> for Context<D> {
    type BBoxError = ContextError;

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match (request.route(), request.guard::<D>().await) {
            (None, _) => Failure((Status::InternalServerError, ContextError::Unconstructible)),
            (Some(route), Success(data)) => Success(Context::new(route.uri.to_string(), data)),
            (_, Failure((status, _))) => Failure((status, ContextError::Unconstructible)),
            (_, Forward(f)) => Forward(f),
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r, T, D: ContextData> FromBBoxRequestAndData<'a, 'r, T> for Context<D> where
    T: FromBBoxData<'a, 'r> + Sync,
    D: FromBBoxRequestAndData<'a, 'r, T>,
{
    type BBoxError = ContextError;

    async fn from_bbox_request_and_data(
        request: BBoxRequest<'a, 'r>,
        data: &T,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match (request.route(), D::from_bbox_request_and_data(request, data).await) {
            (None, _) => Failure((Status::InternalServerError, ContextError::Unconstructible)),
            (Some(route), Success(data)) => Success(Context::new(route.uri.to_string(), data)),
            (_, Failure((status, _))) => Failure((status, ContextError::Unconstructible)),
            (_, Forward(f)) => Forward(f),
        }
    }
}

// Alohomora turns Context into UnprotectedContext before invoking Policy Check.
pub struct UnprotectedContext {
    pub route: String,
    pub data: Box<dyn Any>,
}
impl UnprotectedContext {
    pub(crate) fn from<D: ContextData>(context: Context<D>) -> Self {
        Self {
            route: context.route,
            data: match context.data {
                None => Box::new(Option::<()>::None),
                Some(data) => Box::new(fold(data).unwrap().consume().0),
            }
        }
    }
    pub fn downcast_ref<D: 'static>(&self) -> Option<&D> {
        self.data.downcast_ref()
    }
}
