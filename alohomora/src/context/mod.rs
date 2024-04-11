use std::any::Any;
use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use rocket::http::Status;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use crate::AlohomoraType;
use crate::fold::fold;

// Context Data must satisfy these requirements.
pub trait ContextData : AlohomoraType + for<'a, 'r> FromBBoxRequest<'a, 'r> + Send + 'static {}
impl<D: AlohomoraType + for<'a, 'r> FromBBoxRequest<'a, 'r> + Send + 'static> ContextData for D {}

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
    pub(crate) fn new(route: String, data: D) -> Self {
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
}

// The only way to construct a Context is to get via from a BBoxRequest using below trait.
// This also implies that D has to be constructed that way as well, meaning that any sensitive
// information stored in D (e.g. something from a cookie, like a UserID), will have to originate
// from the BBoxRequest (and thus be in BBox form, at least initially).
#[derive(Debug)]
pub enum ContextError {
    Unconstructible,
}

#[rocket::async_trait]
impl<'a, 'r, D: ContextData> FromBBoxRequest<'a, 'r> for Context<D> {
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