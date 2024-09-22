use crate::config::Config;
use crate::policies::ContextData;
use alohomora::context::UnprotectedContext;
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy};
use alohomora::AlohomoraType;

// ML training policy.
#[schema_policy(table = "ml_training", column = 0)]
#[schema_policy(table = "ml_training", column = 1)]
#[derive(Clone)]
pub struct MLTrainingPolicy {
    consent: bool,
}

impl Policy for MLTrainingPolicy {
    fn name(&self) -> String {
        "MLTrainingPolicy".to_string()
    }

    fn check(&self, context: &UnprotectedContext, _reason: Reason) -> bool {
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let context: &ContextDataOut = context.downcast_ref().unwrap();

        let user: &Option<String> = &context.user;
        let config: &Config = &context.config;

        let user = user.as_ref().unwrap();
        if config.managers.contains(user) && self.consent {
            return true;
        }
        return false;
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<MLTrainingPolicy>() {
            let other = other.specialize::<MLTrainingPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        Ok(MLTrainingPolicy {
            consent: self.consent && p2.consent,
        })
    }
}

impl SchemaPolicy for MLTrainingPolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        MLTrainingPolicy {
            consent: mysql::from_value(row[2].clone()),
        }
    }
}
