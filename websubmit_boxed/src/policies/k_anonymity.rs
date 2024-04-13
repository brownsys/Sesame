use std::cmp;
use std::sync::{Arc, Mutex};
use alohomora::AlohomoraType;
use alohomora::context::{Context, UnprotectedContext};
use alohomora::policy::{AnyPolicy, Policy, PolicyAnd, Reason, schema_policy, SchemaPolicy};
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::policies::ContextData;

// K-anonymity policy.
#[schema_policy(table = "agg_gender", column = 1)]
#[schema_policy(table = "agg_remote", column = 1)]
#[derive(Clone)]
pub struct KAnonymityPolicy {
    count: u64,
}

const MIN_K: u64 = 10;

impl Policy for KAnonymityPolicy {
    fn name(&self) -> String {
        format!("KAnonymityPolicy(count={})", self.count)
    }

    fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        self.count >= MIN_K
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<KAnonymityPolicy>() {
            let other = other.specialize::<KAnonymityPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(
                PolicyAnd::new(
                    AnyPolicy::new(self.clone()),
                    other)))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        Ok(KAnonymityPolicy {
            count: cmp::max(self.count, p2.count),
        })
    }
}

impl SchemaPolicy for KAnonymityPolicy {
    fn from_row(table: &str, row: &Vec<mysql::Value>) -> Self
        where
            Self: Sized,
    {
        KAnonymityPolicy { count: mysql::from_value(row[2].clone()) }
    }
}
