use crate::context::UnprotectedContext;
use crate::policy::{
    AnyPolicyDyn, CheckVisitor, Join, NameVisitor, NoPolicy, Policy, PolicyReflection, Reason,
    Specialize,
};

pub type SpecializationEnum =
    PolicyReflection<'static, Box<dyn AnyPolicyDyn + 'static>, Box<dyn Policy + 'static>>;

// Mutable reflections are policies (they need to be mutable in order to allow joining them with others).
impl Policy for SpecializationEnum {
    fn name(&self) -> String {
        let mut v = NameVisitor {};
        self.postfix_visit_by_ref(&mut v)
    }

    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        let mut v = CheckVisitor::new(context, reason);
        self.postfix_visit_by_ref(&mut v)
    }
}

// Owned Static Reflection Enum is Unjoinable.
impl Join for SpecializationEnum {}

// We can specialize OwnedReflection (that's the whole point!).
impl SpecializationEnum {
    // Visitor pattern.
    pub fn specialize<P: Specialize>(self) -> Result<P, Self> {
        match self {
            Self::NoReflection(pol) => {
                let pol: Box<dyn Policy + 'static> = pol;
                P::specialize_leaf(pol.upgrade_to_any_box()).map_err(Self::Leaf)
            }
            Self::Leaf(b) => P::specialize_leaf(b).map_err(Self::Leaf),
            Self::PolicyAnd(p1, p2) => match P::specialize_and(p1, p2) {
                Ok(p) => Ok(p),
                Err((p1, p2)) => match P::specialize_and(p2, p1) {
                    Ok(p) => Ok(p),
                    Err((mut p2, mut p1)) => {
                        if p1.is_no_policy() {
                            match p2.specialize::<P>() {
                                Ok(p2) => {
                                    return Ok(p2);
                                }
                                Err(e) => {
                                    p2 = Box::new(e);
                                }
                            }
                        }
                        if p2.is_no_policy() {
                            match p1.specialize::<P>() {
                                Ok(p1) => {
                                    return Ok(p1);
                                }
                                Err(e) => {
                                    p1 = Box::new(e);
                                }
                            }
                        }
                        Err(Self::PolicyAnd(p1, p2))
                    }
                },
            },
            Self::PolicyOr(p1, p2) => match P::specialize_or(p1, p2) {
                Ok(p) => Ok(p),
                Err((p1, p2)) => match P::specialize_or(p2, p1) {
                    Ok(p2) => Ok(p2),
                    Err((mut p2, mut p1)) => {
                        match p1.specialize::<P>() {
                            Ok(p1) => {
                                return Ok(p1);
                            }
                            Err(e) => {
                                p1 = Box::new(e);
                            }
                        }
                        match p2.specialize::<P>() {
                            Ok(p2) => {
                                return Ok(p2);
                            }
                            Err(e) => {
                                p2 = Box::new(e);
                            }
                        }
                        Err(Self::PolicyOr(p1, p2))
                    }
                },
            },
            PolicyReflection::PolicyRef(pol, _e) => {
                let pol: Box<dyn Policy + 'static> = pol;
                P::specialize_leaf(pol.upgrade_to_any_box()).map_err(Self::Leaf)
            }
            Self::OptionPolicy(p) => {
                let p = match P::specialize_option(p) {
                    Ok(p) => {
                        return Ok(p);
                    }
                    Err(p) => p,
                };
                match p {
                    None => {
                        let p = Box::new(NoPolicy {});
                        match P::specialize_leaf(p) {
                            Ok(p) => Ok(p),
                            Err(_) => Err(Self::OptionPolicy(None)),
                        }
                    }
                    Some(p) => match p.specialize::<P>() {
                        Ok(p) => Ok(p),
                        Err(e) => Err(Self::OptionPolicy(Some(Box::new(e)))),
                    },
                }
            }
            Self::AnyPolicy(_) => panic!("use normalize() first"),
            Self::TestPolicy(_) => panic!("use normalize() first"),
            PolicyReflection::_Unreachable(inf, _) => match inf {},
        }
    }
}
