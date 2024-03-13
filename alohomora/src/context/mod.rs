use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::outcome::Outcome::{Failure, Forward, Success};

#[derive(Debug)]
pub struct Context<U: 'static, D:'static> {
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

    pub fn new(user: Option<U>, route: String, data: D) -> Self {
        Self {
            user,
            route: route,
            data,
        }
    } 

    /*#[cfg(test)]
    pub(crate) fn new(user: Option<U>, data: D) -> Self {
        Context {
            user,
            route: String::from("test"),
            data,
        }
    }*/
}

#[derive(Debug)]
pub enum ContextError {
    Unconstructible,
}

#[rocket::async_trait]
impl<'a, 'r, U: FromBBoxRequest<'a, 'r>, D: FromBBoxRequest<'a, 'r> + Send> FromBBoxRequest<'a, 'r>
    for Context<U, D>
{
    type BBoxError = ContextError;

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
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
            .and_then(|route| {
                Some(Context {
                    user: user,
                    route: route.uri.to_string(),
                    data: data.unwrap(),
                })
            })
            .into_outcome((Status::InternalServerError, ContextError::Unconstructible))
    }
}
