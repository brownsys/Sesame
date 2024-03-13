use crate::rocket::data::BBoxData;
use crate::rocket::request::BBoxRequest;
use crate::rocket::response::BBoxResponseOutcome;

// The return type of a request's inner lambda handler.
type BBoxFuture<'a> = futures::future::BoxFuture<'a, BBoxResponseOutcome<'a>>;

// Box::new(<BBoxHandlerLambda>) -> BBoxHandler.
pub type BBoxRouteHandlerLambda =
    for<'a, 'r> fn(request: BBoxRequest<'a, 'r>, data: BBoxData<'a>) -> BBoxFuture<'a>;

// Our #[bbox_get(...)] #[bbox_post(...)], etc macros generate a struct with an ::info() function
// that returns an instance of this.
pub struct BBoxRouteInfo {
    pub method: rocket::http::Method,
    pub uri: &'static str,
    pub bbox_handler: BBoxRouteHandlerLambda,
}

// BBoxRoute is just a wrapper around a regular rocket Route.
// It adds a wrapper that acts as a the plain rocket handler,
// the wrapper takes in unprotected clear rocket types, e.g. rocket::request::Request,
// the wrapper then turns them into the BBox versions, e.g. BBoxRequest,
// and then calls the handler specified by the application.
pub struct BBoxRoute {
    pub(crate) route: rocket::route::Route,
}
impl From<BBoxRouteInfo> for BBoxRoute {
    fn from(value: BBoxRouteInfo) -> Self {
        BBoxRoute {
            route: rocket::route::Route::new(
                value.method,
                value.uri,
                BBoxRouteHandlerWrapper::new(value.bbox_handler),
            ),
        }
    }
}
impl From<rocket::fs::FileServer> for BBoxRoute {
    fn from(value: rocket::fs::FileServer) -> Self {
        let mut vec: Vec<rocket::route::Route> = value.into();
        BBoxRoute {
            route: vec.pop().unwrap(),
        }
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
struct BBoxRouteHandlerWrapper {
    bbox_handler: BBoxRouteHandlerLambda,
}
impl BBoxRouteHandlerWrapper {
    pub fn new(bbox_handler: BBoxRouteHandlerLambda) -> Self {
        BBoxRouteHandlerWrapper { bbox_handler }
    }
}
#[rocket::async_trait]
impl rocket::route::Handler for BBoxRouteHandlerWrapper {
    async fn handle<'a>(
        &self,
        request: &'a rocket::request::Request<'_>,
        data: rocket::data::Data<'a>,
    ) -> rocket::route::Outcome<'a> {
        let result_future: BBoxResponseOutcome<'a> = (self.bbox_handler)(BBoxRequest::new(request), BBoxData::new(data)).await;
        match result_future {
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
