use rocket_cors;

use crate::rocket::SesameRoute;

pub fn catch_all_options_routes() -> Vec<SesameRoute> {
    let routes = rocket_cors::catch_all_options_routes();
    routes
        .into_iter()
        .map(|route| SesameRoute { route })
        .collect()
}
