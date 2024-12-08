use crate::config::Config;
use crate::policies::ContextData;
use alohomora::context::UnprotectedContext;
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy};
use alohomora::AlohomoraType;
use crate::policies::User;

// Access control policy.
// #[schema_policy(table = "users", column = 5)] // gender
// #[schema_policy(table = "users", column = 6)] // age
// #[schema_policy(table = "users", column = 7)] // ethnicity

alohomora_policy::access_control_policy!(UserProfilePolicy, ContextData, User,
    [is_not_authenticated, alohomora_policy::never_leaked!()],
    [is_owner || is_admin, alohomora_policy::anything!()]
    (alohomora_policy::never_leaked!());
    User::combine);


#[derive(Clone)]
pub struct OGUserProfilePolicy {
    owner: Option<String>, // even if no owner, admins may access
}

impl Policy for OGUserProfilePolicy {
    fn name(&self) -> String {
        "OGUserProfilePolicy".to_string()
    }

    fn check(&self, context: &UnprotectedContext, _reason: Reason) -> bool {
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let context: &ContextDataOut = context.downcast_ref().unwrap();

        let user: &Option<String> = &context.user;
        let config: &Config = &context.config;

        // I am not an authenticated user. I cannot see any profiles!
        if user.is_none() {
            return false;
        }

        // I am the owner of the profile.
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
        if other.is::<OGUserProfilePolicy>() {
            let other = other.specialize::<OGUserProfilePolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        let comp_owner: Option<String>;
        if self.owner.eq(&p2.owner) {
            comp_owner = self.owner.clone();
        } else {
            comp_owner = None;
        }
        Ok(OGUserProfilePolicy { owner: comp_owner })
    }
}

impl SchemaPolicy for OGUserProfilePolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        OGUserProfilePolicy {
            owner: mysql::from_value(row[0].clone()),
        }
    }
}

