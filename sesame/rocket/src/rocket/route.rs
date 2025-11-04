use crate::rocket::data::PConData;
use crate::rocket::request::PConRequest;
use crate::rocket::response::PConResponseOutcome;

// The return type of a request's inner lambda handler.
type PConFuture<'a> = futures::future::BoxFuture<'a, PConResponseOutcome<'a>>;

// Box::new(<SesameRouteHandlerLambda>) -> sesame route handler.
pub type SesameRouteHandlerLambda =
    for<'a, 'r> fn(request: PConRequest<'a, 'r>, data: PConData<'a>) -> PConFuture<'a>;

// Our #[get(...)] #[post(...)], etc macros generate a struct with an ::info() function
// that returns an instance of this.
pub struct SesameRouteInfo {
    pub method: rocket::http::Method,
    pub uri: &'static str,
    pub handler: SesameRouteHandlerLambda,
}

// SesameRoute is just a wrapper around a regular rocket Route.
// It adds a wrapper that acts as a the plain rocket handler,
// the wrapper takes in unprotected clear rocket types, e.g. rocket::request::Request,
// the wrapper then turns them into the PCon versions, e.g. PConRequest,
// and then calls the handler specified by the application.
pub struct SesameRoute {
    pub(crate) route: rocket::route::Route,
}
impl From<SesameRouteInfo> for SesameRoute {
    fn from(value: SesameRouteInfo) -> Self {
        SesameRoute {
            route: rocket::route::Route::new(
                value.method,
                value.uri,
                SesameRouteHandlerWrapper::new(value.handler),
            ),
        }
    }
}
impl From<rocket::fs::FileServer> for SesameRoute {
    fn from(value: rocket::fs::FileServer) -> Self {
        let mut vec: Vec<rocket::route::Route> = value.into();
        SesameRoute {
            route: vec.pop().unwrap(),
        }
    }
}

// Internal type that makes our Sesame route handlers compatible with rocket
// (which excepts unboxed plain handlers).
// It takes in the unsafe parameters that rocket passes when handling an endpoint
// then, it wraps them in safe PCon types,
// and calls the application-level Sesame endpoint handler, passing the wrapped type.
// the Sesame endpoint handler extracts the needed params, guards, and forms (all PCon-ed),
// and invokes the user provided endpoint handling function, returning its PCon
// result to this internal function.
// Finally, this struct unwraps the PCon result, and returns the unprotected result
// to rocket.
// This last part removes the PCon protection, but this is OK: application code
// cannot access this part, as this handler is passed directly to rocket.
#[derive(Clone)]
struct SesameRouteHandlerWrapper {
    handler: SesameRouteHandlerLambda,
}
impl SesameRouteHandlerWrapper {
    pub fn new(handler: SesameRouteHandlerLambda) -> Self {
        SesameRouteHandlerWrapper { handler }
    }
}
#[rocket::async_trait]
impl rocket::route::Handler for SesameRouteHandlerWrapper {
    async fn handle<'a>(
        &self,
        request: &'a rocket::request::Request<'_>,
        data: rocket::data::Data<'a>,
    ) -> rocket::route::Outcome<'a> {
        let result_future: PConResponseOutcome<'a> =
            (self.handler)(PConRequest::new(request), PConData::new(data)).await;
        match result_future {
            PConResponseOutcome::Success(response) => {
                rocket::outcome::Outcome::Success(response.get_response())
            }
            PConResponseOutcome::Failure(status) => rocket::outcome::Outcome::Failure(status),
            PConResponseOutcome::Forward(data) => {
                rocket::outcome::Outcome::Forward(data.get_data())
            }
        }
    }
}
