use crate::policy::{AnyPolicy, Policy};

pub enum AlohomoraTypePolicy {
    Policy(AnyPolicy),
    NoPolicy,
}

pub fn compose_policies(policy1: AlohomoraTypePolicy, policy2: AlohomoraTypePolicy) -> AlohomoraTypePolicy {
    match (policy1, policy2) {
        (AlohomoraTypePolicy::NoPolicy, policy2) => policy2,
        (policy1, AlohomoraTypePolicy::NoPolicy) => policy1,
        (AlohomoraTypePolicy::Policy(policy1), AlohomoraTypePolicy::Policy(policy2)) =>
            AlohomoraTypePolicy::Policy(policy1.join(policy2).unwrap()),
    }
}