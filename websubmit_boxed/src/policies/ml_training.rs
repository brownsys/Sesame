use crate::config::Config;
use crate::policies::ContextData;
use alohomora::context::UnprotectedContext;
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy};
use alohomora::AlohomoraType;

use super::User;

// ML training policy.
#[schema_policy(table = "ml_training", column = 0)]
#[schema_policy(table = "ml_training", column = 1)]
#[derive(Clone)]
pub struct OGMLTrainingPolicy {
    consent: bool,
}

pub type MLTrainingPolicy = MLTrainingPolicy2;

alohomora_policy::consent_policy!(MLTrainingPolicy2, User, [3, alohomora_policy::anything!()]);

impl Policy for OGMLTrainingPolicy {
    fn name(&self) -> String {
        "OGMLTrainingPolicy".to_string()
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
        if other.is::<OGMLTrainingPolicy>() {
            let other = other.specialize::<OGMLTrainingPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        Ok(OGMLTrainingPolicy {
            consent: self.consent && p2.consent,
        })
    }
}

impl SchemaPolicy for OGMLTrainingPolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        OGMLTrainingPolicy {
            consent: mysql::from_value(row[2].clone()),
        }
    }
}
