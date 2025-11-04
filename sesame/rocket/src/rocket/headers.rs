use crate::policy::FrontendPolicy;
use sesame::pcon::PCon;

pub struct PConHeaderMap<'a, 'r> {
    request: &'a rocket::Request<'r>,
    map: &'a rocket::http::HeaderMap<'r>,
}

impl<'a, 'r> PConHeaderMap<'a, 'r> {
    pub fn new(
        request: &'a rocket::Request<'r>,
        map: &'a rocket::http::HeaderMap<'r>,
    ) -> PConHeaderMap<'a, 'r> {
        PConHeaderMap { request, map }
    }
    pub fn get_one<P: FrontendPolicy>(&self, name: &str) -> Option<PCon<String, P>> {
        self.map
            .get_one(name)
            .map(|token: &str| PCon::new(String::from(token), P::from_request(self.request)))
    }
}
