use crate::policy::{
    AnyPolicyDyn, IsNoPolicy, Join, MutRefReflection, NoPolicy, Policy, PolicyAnd, PolicyDyn,
    SimplePolicy,
};
use crate::testing::TestPolicy;

// SimpleLeafs.
impl<P: SimplePolicy> Join for P {
    fn can_join_with(&mut self, p: &MutRefReflection<'_>) -> bool {
        match p {
            MutRefReflection::Leaf(s) => {
                s.upcast_any().is::<P>() || s.upcast_any().is::<NoPolicy>()
            }
            _ => false,
        }
    }
    fn join_via_reflection(&mut self, p: MutRefReflection<'_>) -> bool {
        match p {
            MutRefReflection::Leaf(other) => {
                let other = other.upcast_mut();
                if other.is::<P>() {
                    let other = other.downcast_mut().unwrap();
                    self.simple_join_direct(other);
                    true
                } else if other.is::<NoPolicy>() {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

// NoPolicy.
impl Join for NoPolicy {
    fn can_join_with(&mut self, p: &MutRefReflection<'_>) -> bool {
        let mut v = IsNoPolicy {};
        p.postfix_visit_by_ref(&mut v)
    }
    fn join_via_reflection(&mut self, p: MutRefReflection<'_>) -> bool {
        self.can_join_with(&p)
    }
}

// PolicyAnd.
impl<P1: Policy, P2: Policy> Join for PolicyAnd<P1, P2> {
    fn can_join_with(&mut self, p: &MutRefReflection<'_>) -> bool {
        let (p1, p2) = self.mut_policies();
        match p {
            MutRefReflection::PolicyAnd(left, right) => {
                (p1.can_join_with(left) && p2.can_join_with(right))
                    || p1.can_join_with(p)
                    || p2.can_join_with(p)
            }
            _ => p1.can_join_with(p) || p2.can_join_with(p),
        }
    }
    fn join_via_reflection(&mut self, p: MutRefReflection<'_>) -> bool {
        // Try to join left with left and right with right.
        let (p1, p2) = self.mut_policies();
        let p = match p {
            MutRefReflection::PolicyAnd(left, right) => {
                if p1.can_join_with(&left) && p2.can_join_with(&right) {
                    if !p1.join_via_reflection(*left) || !p2.join_via_reflection(*right) {
                        panic!("join returned false even though can join returned true");
                    }
                    return true;
                } else {
                    MutRefReflection::PolicyAnd(left, right)
                }
            }
            p => p,
        };
        // Try to join left or then right.
        if p1.can_join_with(&p) {
            if !p1.join_via_reflection(p) {
                panic!("join returned false even though can join returned true");
            }
            true
        } else {
            p2.join_via_reflection(p)
        }
    }
}

// AnyPolicy.
impl<P: PolicyDyn + ?Sized> Join for AnyPolicyDyn<P> {
    fn can_join_with(&mut self, p: &MutRefReflection<'_>) -> bool {
        self.mut_inner().can_join_with(p)
    }
    fn join_via_reflection(&mut self, p: MutRefReflection<'_>) -> bool {
        self.mut_inner().join_via_reflection(p)
    }
}

// Test Policy.
impl<P: Policy> Join for TestPolicy<P> {
    fn can_join_with(&mut self, p: &MutRefReflection<'_>) -> bool {
        self.mut_policy().can_join_with(p)
    }
    fn join_via_reflection(&mut self, p: MutRefReflection<'_>) -> bool {
        self.mut_policy().join_via_reflection(p)
    }
}
