use sesame::testing::TestPolicy;

use crate::policy::FrontendPolicy;

impl<P: FrontendPolicy> FrontendPolicy for TestPolicy<P> {
    fn from_request(request: &rocket::Request<'_>) -> Self {
        TestPolicy::new(P::from_request(request))
    }
    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>,
    ) -> Self {
        TestPolicy::new(P::from_cookie(name, cookie, request))
    }
}
