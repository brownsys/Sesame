use crate::policy::{Policy, AnyPolicyDyn, PolicyDyn};

pub fn compose_policies<P: PolicyDyn + ?Sized>(
    policy1: Result<Option<AnyPolicyDyn<P>>, ()>,
    policy2: Result<Option<AnyPolicyDyn<P>>, ()>,
) -> Result<Option<AnyPolicyDyn<P>>, ()> {
    let policy1 = policy1?;
    let policy2 = policy2?;
    match (policy1, policy2) {
        (None, policy2) => Ok(policy2),
        (policy1, None) => Ok(policy1),
        (Some(policy1), Some(policy2)) => {
            // TODO(babman): Need to work out the technical debt of joins.
            //               We need to either have concrete join return types
            //               or find a way to preserve dyns through the join
            //               with any types.
            //               back up option: manually implement trait combos in dyns.rs
            //               for PolicyAnd and PolicyOrs.
            //               Require them with PolicyDyn somehow.
            // Ok(Some(policy1.join(policy2)?))
            todo!()
        },
    }
}