use serde::Serialize;
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyTrait, NoPolicy, Policy, Reason, Specializable, SpecializationEnum, Specialize};
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

impl<P: Policy + Specializable> Specializable for OptionPolicy<P> {
    fn to_specialization_enum(self) -> SpecializationEnum {
        match self {
            OptionPolicy::NoPolicy => SpecializationEnum::OptionPolicy(None),
            OptionPolicy::Policy(p) => SpecializationEnum::OptionPolicy(
                Some(Box::new(p.to_specialization_enum()))
            ),
        }
    }
    fn to_specialization_enum_box(self: Box<Self>) -> SpecializationEnum {
        self.to_specialization_enum()
    }
}

impl<P: Policy + Specialize> Specialize for OptionPolicy<P> {
    fn specialize_leaf(b: Box<dyn AnyPolicyTrait>) -> Result<Self, Box<dyn AnyPolicyTrait>> {
        let any = b.upcast_any();
        if any.is::<NoPolicy>() {
            Ok(OptionPolicy::NoPolicy)
        } else if any.is::<P>() {
            P::specialize_leaf(b).map(OptionPolicy::Policy)
        } else {
            Err(b)
        }
    }
    fn specialize_option(b: Option<Box<SpecializationEnum>>) -> Result<Self, Option<Box<SpecializationEnum>>> {
        match b {
            None => Ok(OptionPolicy::NoPolicy),
            Some(b) => {
                match b.specialize::<P>() {
                    Ok(p) => Ok(OptionPolicy::Policy(p)),
                    Err(b) => Err(Some(Box::new(b))),
                }
            }
        }
    }
}