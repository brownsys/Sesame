use rocket_cors;

use crate::rocket::BBoxRoute;

pub fn catch_all_options_routes() -> Vec<BBoxRoute> {
    let routes = rocket_cors::catch_all_options_routes();
    routes.into_iter()
        .map(|route| {
            BBoxRoute { route }
        })
        .collect()
}