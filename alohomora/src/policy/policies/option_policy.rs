use std::any::Any;
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyable, Policy, Reason, Unjoinable};

#[derive(Clone)]
pub enum OptionPolicy<P: AnyPolicyable> {
    NoPolicy,
    Policy(P),
}

Unjoinable!(OptionPolicy<P> where P: AnyPolicyable);

impl<P: Any + Policy> Policy for OptionPolicy<P> {
    fn name(&self) -> String {
        match self {
            Self::NoPolicy => format!("OptionPolicy(Empty)"),
            Self::Policy(p) => format!("OptionPolicy({})", p.name()),
        }
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        match self {
            Self::NoPolicy => true,
            Self::Policy(p) => p.check(context, reason),
        }
    }
}
