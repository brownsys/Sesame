use crate::pcon::PCon;
use crate::policy::Policy;

use crate::fold_in::{FoldInAllowed, RuntimeFoldIn};

// Fold PCon from outside vector to inside vector.
impl<T, P: Policy + FoldInAllowed + Clone> PCon<Vec<T>, P> {
    pub fn fold_in(self) -> Vec<PCon<T, P>> {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed");
        }
        let (t, p) = self.consume();
        t.into_iter().map(|t| PCon::new(t, p.clone())).collect()
    }
}

impl<T, E, P: Policy + FoldInAllowed> PCon<Result<T, E>, P> {
    pub fn fold_in(self) -> Result<PCon<T, P>, E> {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed");
        }
        let (t, p) = self.consume();
        Ok(PCon::new(t?, p))
    }
}

impl<T, P: Policy + FoldInAllowed> PCon<Option<T>, P> {
    pub fn fold_in(self) -> Option<PCon<T, P>> {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed");
        }
        let (t, p) = self.consume();
        Some(PCon::new(t?, p))
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::context::UnprotectedContext;
    use crate::fold_in::{FoldInAllowed, RuntimeFoldIn};
    use crate::pcon::PCon;
    use crate::policy::{
        AnyPolicyClone, NoPolicy, OptionPolicy, PolicyAnd, PolicyOr, Reason, RefPolicy,
        SimplePolicy,
    };

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
        fn simple_join_direct(&mut self, _other: &mut Self) {}
    }

    /// These tests ensure that we can fold in on foldable policies, including ones hiding
    /// behind refs, options, etc.
    #[test]
    fn test_fold_in_allowed() {
        let pcon: PCon<Option<u64>, NoPolicy> = PCon::new(Some(10u64), NoPolicy {});
        assert_eq!(true, pcon.policy().can_fold_in());
        let opt: Option<PCon<u64, NoPolicy>> = pcon.fold_in();
        assert_eq!(opt.clone().unwrap(), PCon::new(10u64, NoPolicy {}));
        assert_eq!(opt.unwrap().discard_box(), 10u64);

        let pcon: PCon<Result<u64, ()>, NoPolicy> = PCon::new(Ok(10u64), NoPolicy {});
        assert_eq!(true, pcon.policy().can_fold_in());
        let res: Result<PCon<u64, NoPolicy>, ()> = pcon.fold_in();
        assert_eq!(res.clone().unwrap(), PCon::new(10u64, NoPolicy {}));
        assert_eq!(res.unwrap().discard_box(), 10u64);

        let pcon: PCon<Vec<u64>, NoPolicy> = PCon::new(vec![10u64, 20u64], NoPolicy {});
        assert_eq!(true, pcon.policy().can_fold_in());
        let vec: Vec<PCon<u64, NoPolicy>> = pcon.fold_in();
        assert_eq!(&vec[0], &PCon::new(10u64, NoPolicy {}));
        assert_eq!(&vec[1], &PCon::new(20u64, NoPolicy {}));
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_fold_in_allowed_any_policy() {
        let pcon: PCon<Vec<u64>, NoPolicy> = PCon::new(vec![10u64, 11u64, 12u64], NoPolicy {});
        let pcon = pcon.into_any_policy();
        assert_eq!(true, pcon.policy().can_fold_in());
        let vec: Vec<PCon<u64, AnyPolicyClone>> = pcon.fold_in();
        assert_eq!(vec.len(), 3);
        assert_eq!(
            vec[0].specialize_policy_ref().unwrap(),
            PCon::new(10u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[1].specialize_policy_ref().unwrap(),
            PCon::new(11u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[2].specialize_policy_ref().unwrap(),
            PCon::new(12u64, NoPolicy {}).as_ref()
        );
    }

    #[test]
    fn test_fold_in_allowed_any_policy_complex() {
        let policy = AnyPolicyClone::new(NoPolicy {});
        let refpolicy = RefPolicy::new(&policy);
        let pcon: PCon<Vec<u64>, RefPolicy<AnyPolicyClone>> =
            PCon::new(vec![10u64, 11u64, 12u64], refpolicy);

        assert_eq!(true, pcon.policy().can_fold_in());
        let vec: Vec<PCon<u64, RefPolicy<AnyPolicyClone>>> = pcon.fold_in();
        assert_eq!(vec.len(), 3);
        assert_eq!(
            vec[0].specialize_policy_ref().unwrap(),
            PCon::new(10u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[1].specialize_policy_ref().unwrap(),
            PCon::new(11u64, NoPolicy {}).as_ref()
        );
        assert_eq!(
            vec[2].specialize_policy_ref().unwrap(),
            PCon::new(12u64, NoPolicy {}).as_ref()
        );
    }

    #[test]
    #[should_panic]
    fn test_fold_in_not_allowed_any_policy() {
        let pcon: PCon<Vec<u64>, NoFoldPolicy> = PCon::new(vec![10u64], NoFoldPolicy {});
        let pcon = pcon.into_any_policy();
        let _vec_of_pcon: Vec<PCon<u64, _>> = pcon.fold_in(); // correctly panics
    }

    #[test]
    #[should_panic]
    fn test_fold_in_not_allowed_any_policy_complex() {
        let policy = AnyPolicyClone::new(PolicyAnd::new(NoFoldPolicy {}, NoFoldPolicy {}));
        let refpolicy = RefPolicy::new(&policy);

        let pcon: PCon<Vec<u64>, _> = PCon::new(vec![10u64], refpolicy);
        let _vec_of_pcon: Vec<PCon<u64, _>> = pcon.fold_in(); // correctly panics
    }

    #[test]
    fn test_ref_policy_yes_foldin() {
        let policy = NoPolicy {};
        let pcon = PCon::new(Some(1064), RefPolicy::new(&policy));
        assert_eq!(pcon.policy().can_fold_in(), true);
        let opt = pcon.fold_in();
        assert_eq!(PCon::new(1064, RefPolicy::new(&policy)), opt.unwrap());
    }

    #[test]
    fn test_and_policy_yes_foldin() {
        let pcon_of_vec: PCon<Vec<u64>, PolicyAnd<NoPolicy, NoPolicy>> = PCon::new(
            vec![10u64, 11u64, 12u64],
            PolicyAnd::new(NoPolicy {}, NoPolicy {}),
        );
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<PCon<u64, _>> = pcon_of_vec.fold_in();
        assert_eq!(vec_of_pcon.len(), 3);
    }

    #[test]
    fn test_or_policy_yes_foldin() {
        let pcon_of_vec: PCon<Vec<u64>, PolicyOr<NoPolicy, NoPolicy>> = PCon::new(
            vec![10u64, 11u64, 12u64],
            PolicyOr::new(NoPolicy {}, NoPolicy {}),
        );
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<PCon<u64, _>> = pcon_of_vec.fold_in();
        assert_eq!(vec_of_pcon.len(), 3);
    }

    #[test]
    fn test_option_policy_yes_foldin() {
        let pcon_of_vec: PCon<Vec<u64>, OptionPolicy<NoPolicy>> =
            PCon::new(vec![10u64, 11u64, 12u64], OptionPolicy::Policy(NoPolicy {}));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<PCon<u64, _>> = pcon_of_vec.fold_in();
        assert_eq!(vec_of_pcon.len(), 3);
    }
}
