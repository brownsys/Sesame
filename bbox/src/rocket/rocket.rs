use std::convert::TryInto;
use std::fmt::Display;

use crate::rocket::route::BBoxRouteInfo;

pub struct BBoxRocket<P: rocket::Phase> {
    frontend: rocket::Rocket<P>,
}

impl BBoxRocket<rocket::Build> {
    // Start by calling build.
    pub fn build() -> Self {
        BBoxRocket {
            frontend: rocket::build(),
        }
    }
    // Finish building by launching and awaiting result.
    pub async fn launch(self) -> std::result::Result<(), rocket::Error> {
        self.frontend.launch().await
    }

    pub fn attach<F: rocket::fairing::Fairing>(self, fairing: F) -> Self {
        BBoxRocket {
            frontend: self.frontend.attach(fairing),
        }
    }
    pub fn manage<T: Send + Sync + 'static>(self, state: T) -> Self {
        BBoxRocket {
            frontend: self.frontend.manage(state),
        }
    }
    pub fn mount<'a, B>(self, base: B, routes: Vec<BBoxRouteInfo>) -> Self
    where
        B: TryInto<rocket::http::uri::Origin<'a>> + Clone + Display,
        B::Error: Display,
    {
        let routes: Vec<rocket::route::Route> = routes
            .into_iter()
            .map(|route| route.to_rocket_route())
            .collect();
        BBoxRocket {
            frontend: self.frontend.mount(base, routes),
        }
    }
}
