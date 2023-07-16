use std::boxed::Box;

use crate::rocket::data::BBoxData;
use crate::rocket::request::BBoxRequest;
use crate::rocket::response::BBoxResponseOutcome;

// The return type of a request's inner lambda handler.
type BBoxFuture<'r> = futures::future::BoxFuture<'r, BBoxResponseOutcome<'r>>;

// Box::new(<BBoxHandlerLambda>) -> BBoxHandler.
pub type BBoxHandlerLambda =
    for<'r> fn(request: BBoxRequest<'r, '_>, data: BBoxData<'r>) -> BBoxFuture<'r>;

// Our #[bbox_get(...)] #[bbox_post(...)], etc macros generate an instance of
// this.
pub struct BBoxRouteInfo {
    pub method: rocket::http::Method,
    pub uri: &'static str,
    pub bbox_handler: BBoxHandlerLambda,
}
impl BBoxRouteInfo {
    pub(crate) fn to_rocket_route(self) -> rocket::route::Route {
        rocket::route::Route::new(
            self.method,
            self.uri,
            BBoxRouteWrapper::new(self.bbox_handler),
        )
    }
}

// Internal type that makes our BBox route handlers compatible with rocket
// (which excepts unboxed plain handlers).
// It takes in the unsafe parameters that rocket passes when handling an enpoint
// then, it wraps them in safe BBox types,
// and calls the application-level BBox endpoint handler, passing the wrapped type.
// the BBox endpoint handler extracts the needed params, guards, and forms (all BBoxed),
// and invokes the user provided endpoint handling function, returning its BBoxed
// result to this internal function.
// Finally, this struct unwraps the BBoxed result, and returns the unprotected result
// to rocket.
// This last part removes the BBox protection, but this is OK: application code
// cannot access this part, as this handler is passed directly to rocket.
#[derive(Clone)]
struct BBoxRouteWrapper {
    bbox_handler: BBoxHandlerLambda,
}
impl BBoxRouteWrapper {
    pub fn new(bbox_handler: BBoxHandlerLambda) -> Self {
        BBoxRouteWrapper { bbox_handler }
    }
}
#[rocket::async_trait]
impl rocket::route::Handler for BBoxRouteWrapper {
    async fn handle<'r>(
        &self,
        request: &'r rocket::request::Request<'_>,
        data: rocket::data::Data<'r>,
    ) -> rocket::route::Outcome<'r> {
        let result_future = (self.bbox_handler)(BBoxRequest::new(request), BBoxData::new(data));
        match result_future.await {
            BBoxResponseOutcome::Success(response) => {
                rocket::outcome::Outcome::Success(response.get_response())
            }
            BBoxResponseOutcome::Failure(status) => rocket::outcome::Outcome::Failure(status),
            BBoxResponseOutcome::Forward(data) => {
                rocket::outcome::Outcome::Forward(data.get_data())
            }
        }
    }
}
