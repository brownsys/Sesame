use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::request::{self, FromRequest, Request};

#[derive(Debug)]
pub struct Context<U, D> {
    user: Option<U>,
    route: String,
    data: D,
}

impl<U, D> Context<U, D> {
    pub fn get_user(&self) -> &Option<U> {
        &self.user
    }

    pub fn get_route(&self) -> &str {
        &self.route
    }

    pub fn get_data(&self) -> &D {
        &self.data
    }
}

#[derive(Debug)]
pub enum ContextError {
    Unconstructible,
}

#[rocket::async_trait]
impl<'r, U: FromRequest<'r>, D: FromRequest<'r> + Send> FromRequest<'r> for Context<U, D> {
    type Error = ContextError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let data: Option<D> = match request.guard::<D>().await {
            Success(data) => Some(data),
            Failure(_) => None,
            Forward(_) => None,
        };

        let user: Option<U> = match request.guard::<U>().await {
            Success(user) => Some(user),
            Failure(_) => None,
            Forward(_) => None,
        };

        request
            .route()
            // TODO(babman): clean this up
            .and_then(|route| {
                Some(Context {
                    user: user,
                    route: route.name.as_ref().unwrap().clone().into_owned(),
                    data: data.unwrap(),
                })
            })
            .into_outcome((Status::InternalServerError, ContextError::Unconstructible))
    }
}
