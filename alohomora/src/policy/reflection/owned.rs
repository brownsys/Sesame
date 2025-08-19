use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyTrait, AsLeaf, AsNoReflection, NoPolicy, Policy, PolicyReflection, Reason, Specializable, Specialize};

pub type OwnedReflection<'a> = PolicyReflection<'a, Box<dyn AnyPolicyTrait + 'static>, Box<dyn Policy + 'a>>;

impl AsLeaf for Box<dyn AnyPolicyTrait> {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait + 'static) {
        &**self
    }
}
impl<'a> AsNoReflection<'a> for Box<(dyn Policy + 'a)> {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a) where 'a: 'r {
        &**self
    }
}

// An OwnedReflection is a policy.
impl<'a>  Policy for OwnedReflection<'a> {
    fn name(&self) -> String {
        match self {
            PolicyReflection::NoReflection(pol) => pol.name(),
            PolicyReflection::Leaf(leaf) => leaf.name(),
            PolicyReflection::PolicyAnd(p1, p2) => {
                format!("PolicyAnd({} AND {})", p1.name(), p2.name())
            },
            PolicyReflection::PolicyOr(p1, p2) => {
                format!("PolicyAnd({} AND {})", p1.name(), p2.name())
            },
            PolicyReflection::OptionPolicy(p) => {
                match p {
                    None => String::from("OptionPolicy(Empty)"),
                    Some(p) => format!("OptionPolicy({})", p.name()),
                }
            },
            PolicyReflection::AnyPolicy(_) => panic!("Use .normalize()"),
            PolicyReflection::TestPolicy(_) => panic!("Use .normalize()"),
            PolicyReflection::_Unreachable(inf, _) => match *inf {},
        }
    }

    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        match self {
            PolicyReflection::NoReflection(pol) => pol.check(context, reason),
            PolicyReflection::Leaf(leaf) => leaf.check(context, reason),
            PolicyReflection::PolicyAnd(p1, p2) => {
                p1.check(context, reason.clone()) && p2.check(context, reason)
            },
            PolicyReflection::PolicyOr(p1, p2) => {
                p1.check(context, reason.clone()) || p2.check(context, reason)
            },
            PolicyReflection::OptionPolicy(p) => {
                match p {
                    None => true,
                    Some(p) => p.check(context, reason),
                }
            },
            PolicyReflection::AnyPolicy(_) => panic!("Use .normalize()"),
            PolicyReflection::TestPolicy(_) => panic!("Use .normalize()"),
            PolicyReflection::_Unreachable(inf, _) => match *inf {},
        }
    }
    /*
    fn policy_type_enum(&mut self) -> PolicyTypeEnum<'_> {
        self.deref_mut()
    }
    fn can_join_with(&mut self, p: &PolicyTypeEnum<'_>) -> bool {
        // TODO(babman): make join a similar visitor pattern to specialize so we can reuse it here.
        false
    }
    fn join(&mut self, p: PolicyTypeEnum<'_>) -> bool {
        // TODO(babman): make join a similar visitor pattern to specialize so we can reuse it here.
        false
    }
    */
}
