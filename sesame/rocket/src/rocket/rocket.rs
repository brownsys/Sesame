use std::convert::TryInto;
use std::fmt::Display;

use crate::rocket::route::SesameRoute;

pub struct SesameRocket<P: rocket::Phase> {
    frontend: rocket::Rocket<P>,
}

impl<P: rocket::Phase> SesameRocket<P> {
    pub(crate) fn get(self) -> rocket::Rocket<P> {
        self.frontend
    }
}

impl SesameRocket<rocket::Build> {
    // Start by calling build.
    pub fn build() -> Self {
        SesameRocket {
            frontend: rocket::build(),
        }
    }
    // Finish building by launching and awaiting result.
    pub async fn launch(self) -> Result<(), rocket::Error> {
        self.frontend.launch().await
    }

    pub fn attach<F: rocket::fairing::Fairing>(self, fairing: F) -> Self {
        SesameRocket {
            frontend: self.frontend.attach(fairing),
        }
    }
    pub fn manage<T: Send + Sync + 'static>(self, state: T) -> Self {
        SesameRocket {
            frontend: self.frontend.manage(state),
        }
    }
    pub fn mount<'a, B, R>(self, base: B, routes: R) -> Self
    where
        B: TryInto<rocket::http::uri::Origin<'a>> + Clone + Display,
        B::Error: Display,
        R: Into<Vec<SesameRoute>>,
    {
        let routes: Vec<rocket::route::Route> =
            routes.into().into_iter().map(|route| route.route).collect();
        SesameRocket {
            frontend: self.frontend.mount(base, routes),
        }
    }
    pub fn register<'a, B, C>(self, base: B, catchers: C) -> Self
    where
        B: TryInto<rocket::http::uri::Origin<'a>> + Clone + std::fmt::Display,
        B::Error: std::fmt::Display,
        C: Into<Vec<rocket::Catcher>>,
    {
        SesameRocket {
            frontend: self.frontend.register(base, catchers),
        }
    }
}

// Can turn a single SesameRoute into a vector using into().
impl Into<Vec<SesameRoute>> for SesameRoute {
    fn into(self) -> Vec<SesameRoute> {
        vec![self]
    }
}
