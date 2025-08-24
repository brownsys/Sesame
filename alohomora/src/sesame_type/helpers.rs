use crate::policy::{join_dyn, AnyPolicyDyn, PolicyDyn};

pub fn compose_policies<P: PolicyDyn + ?Sized>(
    policy1: Result<Option<AnyPolicyDyn<P>>, ()>,
    policy2: Result<Option<AnyPolicyDyn<P>>, ()>,
) -> Result<Option<AnyPolicyDyn<P>>, ()> {
    let policy1 = policy1?;
    let policy2 = policy2?;
    match (policy1, policy2) {
        (None, policy2) => Ok(policy2),
        (policy1, None) => Ok(policy1),
        (Some(policy1), Some(policy2)) => Ok(Some(join_dyn(policy1, policy2))),
    }
}
