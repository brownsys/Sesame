use crate::policy::{Policy, AnyPolicy};

pub fn compose_policies(
    policy1: Result<Option<AnyPolicy>, ()>,
    policy2: Result<Option<AnyPolicy>, ()>,
) -> Result<Option<AnyPolicy>, ()> {
    let policy1 = policy1?;
    let policy2 = policy2?;
    match (policy1, policy2) {
        (None, policy2) => Ok(policy2),
        (policy1, None) => Ok(policy1),
        (Some(policy1), Some(policy2)) => Ok(Some(policy1.join(policy2)?)),
    }
}