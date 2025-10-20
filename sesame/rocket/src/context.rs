use rocket::http::Status;

use sesame::context::{Context, ContextData};

use crate::rocket::{
    BBoxRequest, BBoxRequestOutcome, FromBBoxData, FromBBoxRequest, FromBBoxRequestAndData,
};

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
        use rocket::outcome::Outcome::*;

        match (request.route(), request.guard::<D>().await) {
            (None, _) => Failure((Status::InternalServerError, ContextError::Unconstructible)),
            (Some(route), Success(data)) => Success(Context::new(route.uri.to_string(), data)),
            (_, Failure((status, _))) => Failure((status, ContextError::Unconstructible)),
            (_, Forward(f)) => Forward(f),
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r, T, D: ContextData> FromBBoxRequestAndData<'a, 'r, T> for Context<D>
where
    T: FromBBoxData<'a, 'r> + Sync,
    D: FromBBoxRequestAndData<'a, 'r, T>,
{
    type BBoxError = ContextError;

    async fn from_bbox_request_and_data(
        request: BBoxRequest<'a, 'r>,
        data: &T,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        use rocket::outcome::Outcome::*;

        match (
            request.route(),
            D::from_bbox_request_and_data(request, data).await,
        ) {
            (None, _) => Failure((Status::InternalServerError, ContextError::Unconstructible)),
            (Some(route), Success(data)) => Success(Context::new(route.uri.to_string(), data)),
            (_, Failure((status, _))) => Failure((status, ContextError::Unconstructible)),
            (_, Forward(f)) => Forward(f),
        }
    }
}
