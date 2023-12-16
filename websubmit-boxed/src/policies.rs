use std::sync::{Arc, Mutex};
use std::any::Any;

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::State;

use bbox::policy::{Policy, AnyPolicy, PolicyAnd, SchemaPolicy}; //Conjunction};
use bbox::context::Context;

use bbox::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use bbox_derive::schema_policy;


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
#[schema_policy(table = "answers", column = 0)]
#[schema_policy(table = "answers", column = 1)]
#[schema_policy(table = "answers", column = 2)]
#[schema_policy(table = "answers", column = 3)]
#[schema_policy(table = "answers", column = 5)]
// We can add multiple #[schema_policy(...)] definitions
// here to reuse the policy accross tables/columns.
#[derive(Clone)]
pub struct AnswerAccessPolicy {
    owner: Option<String>, // even if no owner, admins may access
    lec_id: Option<u64>,  // no lec_id when Policy for multiple Answers from different lecs
} 

impl AnswerAccessPolicy{
    pub fn new(owner: Option<String>, lec_id: Option<u64>) -> AnswerAccessPolicy { 
        AnswerAccessPolicy { owner: owner, lec_id: lec_id }
    }
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
        match self.owner.clone() {
            Some(owner) => 
                if *user.unbox(context) == owner {
                    return true;
                }
            None => ()
        }
        // I am an admin.
        if config.admins.contains(user.unbox(context)) {
            return true;
        }
        // I am a discussion leader.
        match self.lec_id {
            // if lec_id, is the user the appropriate discussion leader?
            Some(lec_id) => {
                let mut bg = db.lock().unwrap();
                let vec = bg.prep_exec(
                    "SELECT * FROM discussion_leaders WHERE lec = ? AND email = ?",
                    vec![lec_id.into(), user.clone().discard_box().into()], 
                );
                drop(bg);
                vec.len() > 0
            }
            // No lec_id, don't check the discussion leaders
            None => 
                return false, 
        }
    }
    fn name(&self) -> String {
        format!("AnswerAccessPolicy(lec id{:?} for user {:?})", self.lec_id, self.owner) //TODO(corinn) naming conventions?
    }
    fn join(&self, other: bbox::policy::AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<AnswerAccessPolicy>() { //Policies are combinable
            let other = other.specialize::<AnswerAccessPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {                    //Policies must be stacked
            Ok(AnyPolicy::new(
                PolicyAnd::new(
                    AnyPolicy::new(self.clone()),
                    other)))
        }
    }
    fn join_logic(&self, p2: Self) -> Result<Self, ()> {     
        let comp_owner: Option<String>;  
        let comp_lec_id: Option<u64>;
        if self.owner.eq(&p2.owner) {  
           comp_owner = self.owner.clone();
        } else {
            comp_owner = None;
        }
        if self.lec_id.eq(&p2.lec_id) {  
            comp_lec_id = self.lec_id.clone();
        } else {
            comp_lec_id = None;
        }
        Ok(AnswerAccessPolicy{
            owner: comp_owner,
            lec_id: comp_lec_id
        })
    }
}

impl SchemaPolicy for AnswerAccessPolicy {
    fn from_row(row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        AnswerAccessPolicy {
            owner: mysql::from_value(row[0].clone()),
            lec_id: mysql::from_value(row[1].clone()),
        }
    }
}
