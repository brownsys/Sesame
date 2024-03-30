use crate::context::UnprotectedContext;
use crate::policy::{Policy, AnyPolicy, PolicyAnd, Reason};

#[derive(Clone)]
pub enum OptionPolicy<P: Policy + Clone + 'static> {
    NoPolicy,
    Policy(P),
}
impl<P: Policy + Clone + 'static> Policy for OptionPolicy<P> {
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
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<Self>() {
            Ok(AnyPolicy::new(self.join_logic(other.specialize().unwrap())?))
        } else {
            Ok(AnyPolicy::new(PolicyAnd::new(self.clone(), other)))
        }
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        match (self, other) {
            (Self::NoPolicy, other) => Ok(other),
            (_, Self::NoPolicy) => Ok(self.clone()),
            (Self::Policy(p1), Self::Policy(p2)) => Ok(Self::Policy(p1.join_logic(p2)?)),
        }
    }
}