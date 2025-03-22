use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::policies::ContextData;
use alohomora::context::{Context, UnprotectedContext};
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy};
use alohomora::AlohomoraType;
use std::sync::{Arc, Mutex};

// Access control policy.
#[schema_policy(table = "answers", column = 0)] // email
#[schema_policy(table = "answers", column = 1)] // lec
#[schema_policy(table = "answers", column = 2)] // q
#[schema_policy(table = "answers", column = 3)] // answer
#[schema_policy(table = "answers", column = 4)] // submitted_at
#[schema_policy(table = "answers", column = 5)]
// grade, WHY CAN DISCUSSION LEADER SEE GRADE
// We can add multiple #[schema_policy(...)] definitions
// here to reuse the policy across tables/columns.
#[derive(Clone)]
pub struct AnswerAccessPolicy {
    owner: Option<String>, // even if no owner, admins may access
    lec_id: Option<u64>,   // no lec_id when Policy for multiple Answers from different lectures
}

impl AnswerAccessPolicy {
    pub fn new(owner: Option<String>, lec_id: Option<u64>) -> AnswerAccessPolicy {
        AnswerAccessPolicy {
            owner: owner,
            lec_id: lec_id,
        }
    }
}

// Content of answer column can only be accessed by:
//   1. The user who submitted the answer (`user_id == me`);
//   2. The admin(s) (`is me in set<admins>`);
//   3. Any student who is leading discussion for the lecture
//      (`P(me)` alter. `is me in set<P(students)>`);
impl Policy for AnswerAccessPolicy {
    fn name(&self) -> String {
            "AnswerAccessPolicy".to_string()
    }

    fn check(&self, context: &UnprotectedContext, _reason: Reason) -> bool {
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let context: &ContextDataOut = context.downcast_ref().unwrap();

        let user: &Option<String> = &context.user;
        let db: &Arc<Mutex<MySqlBackend>> = &context.db;
        let config: &Config = &context.config;

        // I am not an authenticated user. I cannot see any answers!
        if user.is_none() {
            return false;
        }

        // I am the owner of the answer.
        let user = user.as_ref().unwrap();
        if let Some(owner) = &self.owner {
            if owner == user {
                return true;
            }
        }

        // I am an admin.
        if config.admins.contains(user) {
            return true;
        }

        // I am a discussion leader.
        if let Some(lec_id) = self.lec_id {
            let mut bg = db.lock().unwrap();
            let vec = bg.prep_exec(
                "SELECT * FROM discussion_leaders WHERE lec = ? AND email = ?",
                (lec_id, user),
                Context::empty(),
            );
            return vec.len() > 0;
        }

        return false;
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<AnswerAccessPolicy>() {
            // Policies are combinable
            let other = other.specialize::<AnswerAccessPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            //Policies must be stacked
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
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
        Ok(AnswerAccessPolicy {
            owner: comp_owner,
            lec_id: comp_lec_id,
        })
    }
}

impl SchemaPolicy for AnswerAccessPolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        AnswerAccessPolicy::new(
            mysql::from_value(row[0].clone()),
            mysql::from_value(row[1].clone()),
        )
    }
}
