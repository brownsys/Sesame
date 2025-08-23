use serde::Serialize;
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyTrait, MutRefReflection, NoPolicy, OwnedReflection, Policy, PolicyAnd, Reason, RefReflection, Reflective, Specializable, SpecializationEnum, Specialize};
use crate::Unjoinable;
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub enum OptionPolicy<P: Policy> {
    NoPolicy,
    Policy(P),
}

impl<P: Policy> Policy for OptionPolicy<P> {
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
    Unjoinable!(!Any);
}