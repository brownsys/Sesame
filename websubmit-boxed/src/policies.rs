use std::sync::{Arc, Mutex};

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::State;

use bbox::policy::Policy;
use bbox::context::Context;

use bbox::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use bbox_derive::schema_policy;

use std::any::Any;

use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::User; 

// Custom developer defined payload attached to every context.
pub struct ContextData {
    pub db: Arc<Mutex<MySqlBackend>>,
    pub config: Config,
}

#[derive(Debug)]
pub enum ContextDataError {
    Unconstructible,
}

// Build the custom payload for the context given HTTP request.
#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for ContextData {
    type BBoxError = ContextDataError;

    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let db = match request.guard::<&State<Arc<Mutex<MySqlBackend>>>>().await {
            Success(db) => Some(db.inner().clone()),
            Failure(_) => None,
            Forward(_) => None,
        };

        let config = match request.guard::<&State<Config>>().await {
            Success(config) => Some(config.inner().clone()),
            Failure(_) => None,
            Forward(_) => None,
        };

        request
            .route()
            .and_then(|_| {
                Some(ContextData {
                    db: db.unwrap(),
                    config: config.unwrap(),
                })
            })
            .into_outcome((
                Status::InternalServerError,
                ContextDataError::Unconstructible,
            ))
    }
}

// Access control policy.
#[schema_policy(table = "answers", column = 3)]
// We can add multiple #[schema_policy(...)] definitions
// here to reuse the policy accross tables/columns.
pub struct AnswerAccessPolicy {
    owner: String,
    lec_id: u64,
}

// Content of answer column can only be accessed by:
//   1. The user who submitted the answer (`user_id == me`);
//   2. The admin(s) (`is me in set<admins>`);
//   3. Any student who is leading discussion for the lecture
//      (`P(me)` alter. `is me in set<P(students)>`);

impl Policy for AnswerAccessPolicy {
    fn from_row(row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        AnswerAccessPolicy {
            owner: mysql::from_value(row[0].clone()),
            lec_id: mysql::from_value(row[1].clone()),
        }
    }

    fn check(&self, context: &dyn Any) -> bool {
        let context: &Context<User, ContextData> = context.downcast_ref().unwrap();

        let user = &context.get_user().as_ref().unwrap().user;
        let db = &context.get_data().db;
        let config = &context.get_data().config;

        // user_id == me
        // TODO(babman): context::user should probably not be BBoxed?
        if *user.test_unbox() == self.owner {
            return true;
        }

        // I am an admin.
        if config.admins.contains(user.test_unbox()) {
            return true;
        }

        // I am a discussion leader.
        let mut bg = db.lock().unwrap();
        let vec = bg.prep_exec(
            "SELECT * FROM discussion_leaders WHERE lec = ? AND email = ?",
            vec![self.lec_id.into(), user.discard_box().into()],
        );
        drop(bg);

        vec.len() > 0
    }

    fn name(&self) -> String {
        format!("AnswerAccessPolicy({} for {})", self.lec_id, self.owner) //TODO(corinn) naming conventions?
    }
}
