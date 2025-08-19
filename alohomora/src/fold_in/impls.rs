use crate::bbox::BBox;
use crate::policy::Policy;

use crate::fold_in::{FoldInAllowed, RuntimeFoldIn};

// Fold PCon from outside vector to inside vector.
impl<T, P: Policy + FoldInAllowed + Clone> BBox<Vec<T>, P> {
    pub fn fold_in(self) -> Vec<BBox<T, P>> {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed");
        }
        let (t, p) = self.consume();
        t.into_iter().map(|t| BBox::new(t, p.clone())).collect()
    }
}

impl<T, E, P: Policy + FoldInAllowed> BBox<Result<T, E>, P> {
    pub fn fold_in(self) -> Result<BBox<T, P>, E> {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed");
        }
        let (t, p) = self.consume();
        Ok(BBox::new(t?, p))
    }
}

impl<T, P: Policy + FoldInAllowed> BBox<Option<T>, P> {
    pub fn fold_in(self) -> Option<BBox<T, P>> {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed");
        }
        let (t, p) = self.consume();
        Some(BBox::new(t?, p))
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::bbox::BBox;
    use crate::context::UnprotectedContext;
    use crate::fold_in::{FoldInAllowed, RuntimeFoldIn};
    use crate::policy::{AnyPolicyCC, NoPolicy, OptionPolicy, PolicyAnd, PolicyOr, Reason, RefPolicy, SimplePolicy};

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

    /// These tests ensure that we can fold in on foldable policies, including ones hiding
    /// behind refs, options, etc.
    #[test]
    fn test_fold_in_allowed() {
        let pcon: BBox<Option<u64>, NoPolicy> = BBox::new(Some(10u64), NoPolicy {});
        assert_eq!(true, pcon.policy().can_fold_in());
        let opt: Option<BBox<u64, NoPolicy>> = pcon.fold_in();
        assert_eq!(opt.clone().unwrap(), BBox::new(10u64, NoPolicy {}));
        assert_eq!(opt.unwrap().discard_box(), 10u64);

        let pcon: BBox<Result<u64, ()>, NoPolicy> = BBox::new(Ok(10u64), NoPolicy {});
        assert_eq!(true, pcon.policy().can_fold_in());
        let res: Result<BBox<u64, NoPolicy>, ()> = pcon.fold_in();
        assert_eq!(res.clone().unwrap(), BBox::new(10u64, NoPolicy {}));
        assert_eq!(res.unwrap().discard_box(), 10u64);

        let pcon: BBox<Vec<u64>, NoPolicy> = BBox::new(vec![10u64, 20u64], NoPolicy {});
        assert_eq!(true, pcon.policy().can_fold_in());
        let vec: Vec<BBox<u64, NoPolicy>> = pcon.fold_in();
        assert_eq!(&vec[0], &BBox::new(10u64, NoPolicy {}));
        assert_eq!(&vec[1], &BBox::new(20u64, NoPolicy {}));
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_fold_in_allowed_any_policy() {
        let pcon: BBox<Vec<u64>, NoPolicy> = BBox::new(vec![10u64, 11u64, 12u64], NoPolicy {});
        let pcon = pcon.into_any_policy();
        assert_eq!(true, pcon.policy().can_fold_in());
        let vec: Vec<BBox<u64, AnyPolicyCC>> = pcon.fold_in();
        assert_eq!(vec.len(), 3);
        assert_eq!(
            vec[0].specialize_policy_ref().unwrap(),
            BBox::new(10u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[1].specialize_policy_ref().unwrap(),
            BBox::new(11u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[2].specialize_policy_ref().unwrap(),
            BBox::new(12u64, NoPolicy {}).as_ref()
        );
    }

    #[test]
    fn test_fold_in_allowed_any_policy_complex() {
        let policy = AnyPolicyCC::new(NoPolicy {});
        let refpolicy = RefPolicy::new(&policy);
        let pcon: BBox<Vec<u64>, RefPolicy<AnyPolicyCC>> =
            BBox::new(vec![10u64, 11u64, 12u64], refpolicy);

        assert_eq!(true, pcon.policy().can_fold_in());
        let vec: Vec<BBox<u64, RefPolicy<AnyPolicyCC>>> = pcon.fold_in();
        assert_eq!(vec.len(), 3);
        assert_eq!(
            vec[0].specialize_policy_ref().unwrap(),
            BBox::new(10u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[1].specialize_policy_ref().unwrap(),
            BBox::new(11u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[2].specialize_policy_ref().unwrap(),
            BBox::new(12u64, NoPolicy {}).as_ref()
        );
    }

    #[test]
    #[should_panic]
    fn test_fold_in_not_allowed_any_policy() {
        let pcon: BBox<Vec<u64>, NoFoldPolicy> = BBox::new(vec![10u64], NoFoldPolicy {});
        let pcon = pcon.into_any_policy();
        let _vec_of_pcon: Vec<BBox<u64, _>> = pcon.fold_in(); // correctly panics
    }

    #[test]
    #[should_panic]
    fn test_fold_in_not_allowed_any_policy_complex() {
        let policy = AnyPolicyCC::new(PolicyAnd::new(NoFoldPolicy {}, NoFoldPolicy {}));
        let refpolicy = RefPolicy::new(&policy);

        let pcon: BBox<Vec<u64>, _> = BBox::new(vec![10u64], refpolicy);
        let _vec_of_pcon: Vec<BBox<u64, _>> = pcon.fold_in(); // correctly panics
    }

    #[test]
    fn test_ref_policy_yes_foldin() {
        let policy = NoPolicy {};
        let pcon = BBox::new(Some(1064), RefPolicy::new(&policy));
        assert_eq!(pcon.policy().can_fold_in(), true);
        let opt = pcon.fold_in();
        assert_eq!(BBox::new(1064, RefPolicy::new(&policy)), opt.unwrap());
    }

    #[test]
    fn test_and_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, PolicyAnd<NoPolicy, NoPolicy>> = BBox::new(
            vec![10u64, 11u64, 12u64],
            PolicyAnd::new(NoPolicy {}, NoPolicy {}),
        );
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in();
        assert_eq!(vec_of_pcon.len(), 3);
    }

    #[test]
    fn test_or_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, PolicyOr<NoPolicy, NoPolicy>> = BBox::new(
            vec![10u64, 11u64, 12u64],
            PolicyOr::new(NoPolicy {}, NoPolicy {}),
        );
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in();
        assert_eq!(vec_of_pcon.len(), 3);
    }

    #[test]
    fn test_option_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, OptionPolicy<NoPolicy>> =
            BBox::new(vec![10u64, 11u64, 12u64], OptionPolicy::Policy(NoPolicy {}));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in();
        assert_eq!(vec_of_pcon.len(), 3);
    }
}
