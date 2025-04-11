use crate::config::Config;
use crate::policies::ContextData;
use alohomora::context::UnprotectedContext;
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy};
use alohomora::AlohomoraType;

// Aggregate access policy.
#[schema_policy(table = "agg_gender", column = 1)]
#[schema_policy(table = "agg_remote", column = 1)]
#[derive(Clone)]
pub struct AggregateAccessPolicy {
    sensitive: bool,
}

const SENSITIVE_TABLES: &'static [&'static str] = &["agg_gender"];

impl Policy for AggregateAccessPolicy {
    fn name(&self) -> String {
        "AggregateAccessPolicy".to_string()
    }

    fn check(&self, context: &UnprotectedContext, _reason: Reason) -> bool {
        true
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<AggregateAccessPolicy>() {
            let other = other.specialize::<AggregateAccessPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        Ok(AggregateAccessPolicy {
            sensitive: self.sensitive || p2.sensitive,
        })
    }
}

impl SchemaPolicy for AggregateAccessPolicy {
    fn from_row(table: &str, _row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        AggregateAccessPolicy {
            sensitive: SENSITIVE_TABLES.contains(&table),
        }
    }
}
