use std::sync::{Arc, Mutex};

use rocket::State;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::request::{self, FromRequest, Request};

use bbox::BBox;
use bbox::context::Context;
use bbox::policy::{Policy, PolicyFactory};

use std::any::Any;

use crate::User;
use crate::backend::MySqlBackend;
use crate::config::Config;

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
impl<'r> FromRequest<'r> for ContextData {
    type Error = ContextDataError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let db = match request.guard::<&State<Arc<Mutex<MySqlBackend>>>>().await {
            Success(db) => { Some(db.inner().clone()) }
            Failure(_) => { None }
            Forward(_) => { None }
        };

        let config = match request.guard::<&State<Config>>().await {
            Success(config) => { Some(config.inner().clone()) }
            Failure(_) => { None }
            Forward(_) => { None }
        };

        request.route()
            .and_then(|route| Some(ContextData { db: db.unwrap(), config: config.unwrap() }))
            .into_outcome((Status::InternalServerError, ContextDataError::Unconstructible))
    }
}


// Access control policy.
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
    fn check(&self, context: &dyn Any) -> bool {
        let context: &Context<User, ContextData> = context.downcast_ref().unwrap();
    
        let user = &context.get_user().as_ref().unwrap().user;
        let db = &context.get_data().db;
        let config = &context.get_data().config;

        // user_id == me
        // TODO(babman): context::user should probably not be BBoxed?
        if *user.internal_unbox() == self.owner {
          return true;
        }

        // I am an admin.
        if config.admins.contains(user.internal_unbox()) {
          return true;
        }

        // I am a discussion leader.
        let mut bg = db.lock().unwrap();
        let vec = bg.prep_exec(
            "SELECT * FROM discussion_leaders WHERE lec = ? AND email = ?",
            vec![self.lec_id.into(), user.into()]
        );
        drop(bg);

        vec.len() > 0
    }
}

// TODO(babman): replace this with macro and do not use a standalone factory.
pub struct AnswerAccessPolicyFactory {}

impl PolicyFactory for AnswerAccessPolicyFactory {
  fn create(&self, row: &Vec<mysql::Value>) -> Box<dyn Policy> {
    Box::new(AnswerAccessPolicy {
        owner: mysql::from_value(row[0].clone()),
        lec_id: mysql::from_value(row[1].clone())
    })
  }
}
