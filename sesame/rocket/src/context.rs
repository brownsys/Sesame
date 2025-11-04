use rocket::http::Status;

use sesame::context::{Context, ContextData};

use crate::rocket::{
    FromPConData, FromPConRequest, FromPConRequestAndData, PConRequest, PConRequestOutcome,
};

// The only way to construct a Context is to get via from a PConRequest using below trait.
// This also implies that D has to be constructed that way as well, meaning that any sensitive
// information stored in D (e.g. something from a cookie, like a UserID), will have to originate
// from the PConRequest (and thus be in PCon form, at least initially).
#[derive(Debug)]
pub enum ContextError {
    Unconstructible,
}

// Context can be constructed from just the request, or the request and the form data, depending on
// what the developer chooses for D.
#[rocket::async_trait]
impl<'a, 'r, D: ContextData + FromPConRequest<'a, 'r>> FromPConRequest<'a, 'r> for Context<D> {
    type PConError = ContextError;

    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
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
impl<'a, 'r, T, D: ContextData> FromPConRequestAndData<'a, 'r, T> for Context<D>
where
    T: FromPConData<'a, 'r> + Sync,
    D: FromPConRequestAndData<'a, 'r, T>,
{
    type PConError = ContextError;

    async fn from_pcon_request_and_data(
        request: PConRequest<'a, 'r>,
        data: &T,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        use rocket::outcome::Outcome::*;

        match (
            request.route(),
            D::from_pcon_request_and_data(request, data).await,
        ) {
            (None, _) => Failure((Status::InternalServerError, ContextError::Unconstructible)),
            (Some(route), Success(data)) => Success(Context::new(route.uri.to_string(), data)),
            (_, Failure((status, _))) => Failure((status, ContextError::Unconstructible)),
            (_, Forward(f)) => Forward(f),
        }
    }
}
