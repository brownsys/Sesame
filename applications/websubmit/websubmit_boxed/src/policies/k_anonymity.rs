use alohomora::context::UnprotectedContext;
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy};
use std::cmp;

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
        "KAnonymityPolicy".to_string()
    }

    fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        self.count >= MIN_K
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<KAnonymityPolicy>() {
            let other = other.specialize::<KAnonymityPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        Ok(KAnonymityPolicy {
            count: cmp::min(self.count, p2.count),
        })
    }
}

impl SchemaPolicy for KAnonymityPolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        KAnonymityPolicy {
            count: mysql::from_value(row[2].clone()),
        }
    }
}
