use sesame::policy::{NoPolicy, Policy, PolicyAnd, PolicyOr};

// Front end policy can be constructed from HTTP requests and from cookies.
pub trait FrontendPolicy: Policy {
    fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
    where
        Self: Sized;

    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>,
    ) -> Self
    where
        Self: Sized;
}

// Impl FrontendPolicy for some policy containers
impl FrontendPolicy for NoPolicy {
    fn from_request(_request: &rocket::Request<'_>) -> Self {
        Self {}
    }

    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a rocket::http::Cookie<'static>,
        _request: &'a rocket::Request<'r>,
    ) -> Self {
        Self {}
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyAnd<P1, P2> {
    fn from_request(request: &rocket::Request<'_>) -> Self {
        PolicyAnd::new(P1::from_request(request), P2::from_request(request))
    }
    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>,
    ) -> Self {
        PolicyAnd::new(
            P1::from_cookie(name, cookie, request),
            P2::from_cookie(name, cookie, request),
        )
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyOr<P1, P2> {
    fn from_request(request: &rocket::Request<'_>) -> Self {
        PolicyOr::new(P1::from_request(request), P2::from_request(request))
    }
    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>,
    ) -> Self {
        PolicyOr::new(
            P1::from_cookie(name, cookie, request),
            P2::from_cookie(name, cookie, request),
        )
    }
}
