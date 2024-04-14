use crate::bbox::BBox;
use crate::policy::FrontendPolicy;

pub struct BBoxHeaderMap<'a, 'r> {
    request: &'a rocket::Request<'r>,
    map: &'a rocket::http::HeaderMap<'r>,
}

impl<'a, 'r> BBoxHeaderMap<'a, 'r> {
    pub fn new(request: &'a rocket::Request<'r>, map: &'a rocket::http::HeaderMap<'r>) -> BBoxHeaderMap<'a, 'r> {
        BBoxHeaderMap { request, map }
    }
    pub fn get_one<P: FrontendPolicy>(&self, name: &str) -> Option<BBox<String, P>> {
        self.map
            .get_one(name)
            .map(|token: &str| BBox::new(String::from(token), P::from_request(self.request)))
    }
}