use crate::config::Config;
use crate::policies::ContextData;
use alohomora::context::UnprotectedContext;
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy};
use alohomora::AlohomoraType;

use super::User;

// ML training policy.
#[schema_policy(table = "employers_release", column = 0)]
#[schema_policy(table = "employers_release", column = 1)]
#[derive(Clone)]
pub struct OGEmployersReleasePolicy {
    consent: bool,
}

pub type EmployersReleasePolicy = EmployersReleasePolicy2;

alohomora_policy::consent_policy!(EmployersReleasePolicy2, User, [2, alohomora_policy::anything!()]);


impl Policy for OGEmployersReleasePolicy {
    fn name(&self) -> String {
        "OGEmployersReleasePolicy".to_string()
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
        if other.is::<OGEmployersReleasePolicy>() {
            let other = other.specialize::<OGEmployersReleasePolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        Ok(OGEmployersReleasePolicy {
            consent: self.consent && p2.consent,
        })
    }
}

impl SchemaPolicy for OGEmployersReleasePolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        OGEmployersReleasePolicy {
            consent: mysql::from_value(row[2].clone()),
        }
    }
}
