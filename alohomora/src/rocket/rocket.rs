use std::convert::TryInto;
use std::fmt::Display;

use crate::rocket::route::BBoxRoute;

pub struct BBoxRocket<P: rocket::Phase> {
    frontend: rocket::Rocket<P>,
}

impl<P: rocket::Phase> BBoxRocket<P> {
    pub(crate) fn get(self) -> rocket::Rocket<P> {
        self.frontend
    }
}

impl BBoxRocket<rocket::Build> {
    // Start by calling build.
    pub fn build() -> Self {
        BBoxRocket {
            frontend: rocket::build(),
        }
    }
    // Finish building by launching and awaiting result.
    pub async fn launch(self) -> Result<(), rocket::Error> {
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
    pub fn mount<'a, B, R>(self, base: B, routes: R) -> Self
    where
        B: TryInto<rocket::http::uri::Origin<'a>> + Clone + Display,
        B::Error: Display,
        R: Into<Vec<BBoxRoute>>,
    {
        let routes: Vec<rocket::route::Route> =
            routes.into().into_iter().map(|route| route.route).collect();
        BBoxRocket {
            frontend: self.frontend.mount(base, routes),
        }
    }
    pub fn register<'a, B, C>(self, base: B, catchers: C) -> Self
        where B: TryInto<rocket::http::uri::Origin<'a>> + Clone + std::fmt::Display,
              B::Error: std::fmt::Display,
              C: Into<Vec<rocket::Catcher>>
    {
        BBoxRocket {
            frontend: self.frontend.register(base, catchers),
        }
    }
}

// Can turn a single BBoxRoute into a vector using into().
impl Into<Vec<BBoxRoute>> for BBoxRoute {
    fn into(self) -> Vec<BBoxRoute> {
        vec![self]
    }
}
