use alohomora::AlohomoraType;
use alohomora::context::UnprotectedContext;
use alohomora::policy::{AnyPolicy, Policy, PolicyAnd, Reason, schema_policy, SchemaPolicy};
use crate::config::Config;
use crate::policies::ContextData;

// ML training policy.
#[schema_policy(table = "employers_release", column = 0)]
#[schema_policy(table = "employers_release", column = 1)]
#[derive(Clone)]
pub struct EmployersReleasePolicy {
    consent: bool,
}

impl Policy for EmployersReleasePolicy {
    fn name(&self) -> String {
        format!("EmployersReleasePolicy(consent={})", self.consent)
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
        if other.is::<EmployersReleasePolicy>() {
            let other = other.specialize::<EmployersReleasePolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(
                PolicyAnd::new(
                    AnyPolicy::new(self.clone()),
                    other)))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        Ok(EmployersReleasePolicy {
            consent: self.consent && p2.consent,
        })
    }
}

impl SchemaPolicy for EmployersReleasePolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
        where
            Self: Sized,
    {
        EmployersReleasePolicy { consent: mysql::from_value(row[2].clone()) }
    }
}
