use std::any::Any;
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyBB, AnyPolicyable, Policy, Reason};

#[derive(Clone)]
pub enum OptionPolicy<P: AnyPolicyable> {
    NoPolicy,
    Policy(P),
}
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
    fn join(&self, _other: AnyPolicyBB) -> Result<AnyPolicyBB, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> {
        todo!()
    }
}
