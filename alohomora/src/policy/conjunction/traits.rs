use crate::policy::{AnyPolicyBB, AnyPolicyDyn, AnyPolicyable, MutRefReflection, NoPolicy, Policy, PolicyAnd, PolicyDyn, PolicyDynRelation, Reflective, SimplePolicy};
use crate::testing::TestPolicy;

// This trait marks that a policy type is safe to join with any instance of the same type.
pub trait ReflexiveJoin : Policy {
    fn reflexive_join(&mut self, other: &mut Self);
}

impl<P: SimplePolicy> ReflexiveJoin for P {
    fn reflexive_join(&mut self, other: &mut Self) {
        self.simple_join_direct(other)
    }
}
impl ReflexiveJoin for NoPolicy {
    fn reflexive_join(&mut self, _other: &mut Self) {}
}
impl<P1: ReflexiveJoin, P2: ReflexiveJoin> ReflexiveJoin for PolicyAnd<P1, P2> {
    fn reflexive_join(&mut self, other: &mut Self) {
        let (p1, p2) = self.mut_policies();
        let (op1, op2) = other.mut_policies();
        p1.reflexive_join(op1);
        p2.reflexive_join(op2);
    }
}
impl<P: ReflexiveJoin> ReflexiveJoin for TestPolicy<P> {
    fn reflexive_join(&mut self, other: &mut Self) {
        self.mut_policy().reflexive_join(other.mut_policy());
    }
}

// Trait for specifying how to join things.
pub trait Join {
    // Whether we can join with the given policy description.
    fn can_join_with(&mut self, p: &MutRefReflection<'_>) -> bool {
        false
    }
    // Join in place.
    fn join_via_reflection(&mut self, p: MutRefReflection<'_>) -> bool {
        false
    }

}
