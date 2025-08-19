use crate::policy::{AnyPolicyDyn, OptionPolicy, Policy, PolicyAnd, PolicyDyn, PolicyOr, RefPolicy};
use crate::testing::TestPolicy;

// Every type (including policy types) are FoldInAllowed by default
// App developers must implement !FoldInAllowed on their types to disable it, e.g.
// using #[derive(NoFoldIn)].
pub auto trait FoldInAllowed {}

// Need to manually implement this for AnyPolicy due to AnyPolicy using a dyn trait object.
impl<P: PolicyDyn + ?Sized> FoldInAllowed for AnyPolicyDyn<P> {}

// Marks which types are not a policy container.
// Needed for specialization of RuntimeFoldIn to work.
pub(crate) auto trait NotAPolicyContainer {}
impl<P: PolicyDyn + ?Sized> !NotAPolicyContainer for AnyPolicyDyn<P> {}
impl<P: Policy> !NotAPolicyContainer for TestPolicy<P> {}
impl<'a, P: Policy + ?Sized> !NotAPolicyContainer for RefPolicy<'a, P> {}
impl<P1: Policy, P2: Policy> !NotAPolicyContainer for PolicyAnd<P1, P2> {}
impl<P1: Policy, P2: Policy> !NotAPolicyContainer for PolicyOr<P1, P2> {}
impl<P: Policy + Clone + 'static> !NotAPolicyContainer for OptionPolicy<P> {}

// AnyPolicy requires a runtime check for whether folding in is allowed or not due to
// type erasure.
pub(crate) trait RuntimeFoldIn {
    fn can_fold_in(&self) -> bool;
}

impl<P: Policy + ?Sized> RuntimeFoldIn for P {
    default fn can_fold_in(&self) -> bool {
        panic!("this should be unreachable");
    }
}

impl<P: Policy + NotAPolicyContainer + ?Sized> RuntimeFoldIn for P {
    default fn can_fold_in(&self) -> bool {
        false
    }
}

impl<P: Policy + NotAPolicyContainer + ?Sized + FoldInAllowed> RuntimeFoldIn for P {
    fn can_fold_in(&self) -> bool {
        true
    }
}

// Manually implement RuntimeFoldIn for types that are !NotAPolicyContainer.
impl<P: PolicyDyn + ?Sized> RuntimeFoldIn for AnyPolicyDyn<P> {
    fn can_fold_in(&self) -> bool {
        self.inner().can_fold_in_erased()
    }
}
impl<P: 'static + Policy + Clone> RuntimeFoldIn for TestPolicy<P> {
    fn can_fold_in(&self) -> bool {
        self.policy().can_fold_in()
    }
}
impl<'a, P: Policy + ?Sized> RuntimeFoldIn for RefPolicy<'a, P> {
    fn can_fold_in(&self) -> bool {
        self.policy().can_fold_in()
    }
}
impl<P1: Policy, P2: Policy> RuntimeFoldIn for PolicyAnd<P1, P2> {
    fn can_fold_in(&self) -> bool {
        self.policy1().can_fold_in() && self.policy2().can_fold_in()
    }
}
impl<P1: Policy, P2: Policy> RuntimeFoldIn for PolicyOr<P1, P2> {
    fn can_fold_in(&self) -> bool {
        self.policy1().can_fold_in() && self.policy2().can_fold_in()
    }
}
impl<P: Policy + Clone + 'static> RuntimeFoldIn for OptionPolicy<P> {
    fn can_fold_in(&self) -> bool {
        match self {
            OptionPolicy::NoPolicy => true,
            OptionPolicy::Policy(p) => p.can_fold_in(),
        }
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::context::UnprotectedContext;
    use crate::fold_in::{FoldInAllowed, RuntimeFoldIn};
    use crate::policy::{AnyPolicyBB, AnyPolicyCC, NoPolicy, OptionPolicy, PolicyAnd, PolicyOr, Reason, RefPolicy, SimplePolicy};
    use crate::testing::TestPolicy;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct NoFoldPolicy {}

    impl !FoldInAllowed for NoFoldPolicy {}

    impl SimplePolicy for NoFoldPolicy {
        fn simple_name(&self) -> String {
            String::from("NoFoldPolicy")
        }
        fn simple_check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
            true
        }
        fn simple_join_direct(&mut self, other: &mut Self) {}
    }

    /// These tests ensure that !FoldInAllowed is correctly propagated.
    #[test]
    fn test_raw_policy_no_fold_in() {
        assert_impl_any!(NoPolicy: FoldInAllowed);
        assert_not_impl_any!(NoFoldPolicy: FoldInAllowed);
        assert_eq!(NoPolicy {}.can_fold_in(), true);
    }

    #[test]
    fn test_any_policy_no_fold_in() {
        let any = AnyPolicyBB::new(NoPolicy {});
        assert_eq!(any.can_fold_in(), true);
        let any = AnyPolicyCC::new(NoPolicy {});
        assert_eq!(any.can_fold_in(), true);
        let any = AnyPolicyBB::new(NoFoldPolicy {});
        assert_eq!(any.can_fold_in(), false);
        let any = AnyPolicyCC::new(NoFoldPolicy {});
        assert_eq!(any.can_fold_in(), false);
    }

    #[test]
    fn test_test_policy_no_fold_in() {
        assert_impl_any!(TestPolicy<NoPolicy>: FoldInAllowed);
        assert_not_impl_any!(TestPolicy<NoFoldPolicy>: FoldInAllowed);
        assert_eq!(TestPolicy::new(NoPolicy {}).can_fold_in(), true);
        assert_eq!(TestPolicy::new(NoFoldPolicy {}).can_fold_in(), false);
    }

    #[test]
    fn test_ref_policy_no_foldin() {
        assert_impl_any!(RefPolicy<NoPolicy>: FoldInAllowed);
        assert_not_impl_any!(RefPolicy<NoFoldPolicy>: FoldInAllowed);

        let policy = NoPolicy {};
        let refpolicy = RefPolicy::new(&policy);
        assert_eq!(refpolicy.can_fold_in(), true);

        let policy = NoFoldPolicy {};
        let refpolicy = RefPolicy::new(&policy);
        assert_eq!(refpolicy.can_fold_in(), false);
    }

    #[test]
    fn test_and_policy_no_foldin() {
        assert_impl_any!(PolicyAnd<NoPolicy, NoPolicy>: FoldInAllowed);
        assert_not_impl_any!(PolicyAnd<NoFoldPolicy, NoFoldPolicy>: FoldInAllowed);
        assert_eq!(PolicyAnd::new(NoPolicy {}, NoPolicy {}).can_fold_in(), true);
        assert_eq!(
            PolicyAnd::new(NoFoldPolicy {}, NoFoldPolicy {}).can_fold_in(),
            false
        );
    }

    #[test]
    fn test_or_policy_no_foldin() {
        assert_impl_any!(PolicyOr<NoPolicy, NoPolicy>: FoldInAllowed);
        assert_not_impl_any!(PolicyOr<NoFoldPolicy, NoFoldPolicy>: FoldInAllowed);
        assert_eq!(PolicyOr::new(NoPolicy {}, NoPolicy {}).can_fold_in(), true);
        assert_eq!(
            PolicyOr::new(NoFoldPolicy {}, NoFoldPolicy {}).can_fold_in(),
            false
        );
    }

    #[test]
    fn test_option_policy_no_foldin() {
        assert_impl_any!(OptionPolicy<NoPolicy>: FoldInAllowed);
        assert_not_impl_any!(OptionPolicy<NoFoldPolicy>: FoldInAllowed);
        assert_eq!(OptionPolicy::Policy(NoPolicy {}).can_fold_in(), true);
        assert_eq!(OptionPolicy::Policy(NoFoldPolicy {}).can_fold_in(), false);
    }
}
