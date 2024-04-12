use std::sync::{Arc, Mutex};
use alohomora::AlohomoraType;
use alohomora::context::{Context, UnprotectedContext};
use alohomora::policy::{AnyPolicy, Policy, PolicyAnd, Reason, schema_policy, SchemaPolicy};
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::policies::ContextData;

// Access control policy.
#[schema_policy(table = "users", column = 5)] // gender
#[schema_policy(table = "users", column = 6)] // age
#[schema_policy(table = "users", column = 7)] // ethnicity
#[schema_policy(table = "agg_gender", column = 0)]
// We can add multiple #[schema_policy(...)] definitions
// here to reuse the policy across tables/columns.
#[derive(Clone)]
pub struct UserProfilePolicy {
    owner: Option<String>, // even if no owner, admins may access
}

impl Policy for UserProfilePolicy {
    fn name(&self) -> String {
        format!("UserProfilePolicy(for user {:?})", self.owner)
    }

    fn check(&self, context: &UnprotectedContext, _reason: Reason) -> bool {
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let context: &ContextDataOut = context.downcast_ref().unwrap();

        let user: &Option<String> = &context.user;
        let config: &Config = &context.config;
        return false;

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

        return false;
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<UserProfilePolicy>() { // Policies are combinable
            let other = other.specialize::<UserProfilePolicy>().unwrap();
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
        if self.owner.eq(&p2.owner) {
            comp_owner = self.owner.clone();
        } else {
            comp_owner = None;
        }
        Ok(UserProfilePolicy{
            owner: comp_owner,
        })
    }
}

impl SchemaPolicy for UserProfilePolicy {
    fn from_row(table: &str, row: &Vec<mysql::Value>) -> Self
        where
            Self: Sized,
    {
        if table == "users" {
            UserProfilePolicy { owner: mysql::from_value(row[0].clone()) }
        } else {
            UserProfilePolicy { owner: None }
        }
    }
}
