use crate::policy::Policy;
use crate::bbox::BBox; 

pub auto trait FoldInAllowed {}

pub trait RuntimeFoldIn {
    fn can_fold_in(&self) -> bool;
}

impl<P: Policy + ?Sized> RuntimeFoldIn for P {
    default fn can_fold_in(&self) -> bool {
        false
    }
}

impl<P: Policy + FoldInAllowed + ?Sized> RuntimeFoldIn for P {
    default fn can_fold_in(&self) -> bool {
        true
    }
}

/// Trait to move PCons from outside a container to inside
pub trait FoldIn<T, P, E> 
where
    P: Policy + FoldInAllowed
{
    type Output;

    fn fold_in(self) -> Self::Output;
}

// Fold PCon from outside vector to inside vector.
impl<T, P> FoldIn<T, P, ()> for BBox<Vec<T>, P> 
where
    P: Policy + FoldInAllowed + RuntimeFoldIn + Clone,
{
    type Output = Vec<BBox<T, P>>;

    fn fold_in(self) -> Self::Output {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed"); 
        }
        let (t, p) = self.consume();
        t.into_iter().map(|t| BBox::new(t, p.clone())).collect()
    }
}

impl<T, E, P> FoldIn<T, P, E> for BBox<Result<T, E>, P> 
where
    P: Policy + FoldInAllowed + RuntimeFoldIn
{
    type Output = Result<BBox<T, P>, E>;

    fn fold_in(self) -> Self::Output {
        if !self.policy().can_fold_in() {
            panic!("fold_in called on policy with folding disallowed"); 
        }
        let (t, p) = self.consume();
        Ok(BBox::new(t?, p))
    }
}

impl<T, P> FoldIn<T, P, ()> for BBox<Option<T>, P> 
where
    P: Policy + FoldInAllowed + RuntimeFoldIn
{
    type Output = Option<BBox<T, P>>;

    fn fold_in(self) -> Self::Output {
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
    use crate::policy::{AnyPolicy, NoPolicy, Policy, RefPolicy, PolicyAnd, PolicyOr, OptionPolicy, Reason};
    use crate::bbox::BBox; 
    use crate::context::UnprotectedContext; 
    use crate::pure::PrivacyPureRegion; 
    use crate::fold_in::{FoldIn, FoldInAllowed, RuntimeFoldIn};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct NoFoldPolicy {
    }

    impl !FoldInAllowed for NoFoldPolicy{}

    impl Policy for NoFoldPolicy {
        fn name(&self) -> String {
            String::from("NoFoldPolicy")
        }
        fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
            true
        }
        fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
            Ok(AnyPolicy::new(self.clone()))
        }
        fn join_logic(&self, _other: Self) -> Result<Self, ()> {
            Ok(NoFoldPolicy {})
        }
    } 

    /// These tests (correctly) fail to compile!
    /*  #[test]
    fn test_raw_policy_no_fold_in() {
        let pcon_of_vec: BBox<Vec<u64>, NoFoldPolicy>  = BBox::new(vec![10u64], NoFoldPolicy { });
        let vec_of_pcon: Vec<BBox<u64, NoFoldPolicy>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.first().unwrap(), &BBox::new(10u64, NoFoldPolicy { }));
    } 
    */

    /*
    #[test]
    fn test_ref_policy_no_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, NoFoldPolicy>  = BBox::new(vec![10u64, 11u64, 12u64], NoFoldPolicy {});
        let pcon_of_vec: BBox<&Vec<u64>, RefPolicy<'_, NoFoldPolicy>> = pcon_of_vec.as_ref();
        let pcon_of_vec: BBox<Vec<u64>, _> = pcon_of_vec.into_ppr(PrivacyPureRegion::new(|v: &Vec<u64>| v.clone()));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.len(), 3);
    }
    */
    /* 
    #[test]
    fn test_and_policy_no_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, PolicyAnd<NoFoldPolicy, NoFoldPolicy>> = BBox::new(vec![10u64, 11u64, 12u64], PolicyAnd::new(NoFoldPolicy {}, NoFoldPolicy {}));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), false);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in(); 
    } 
    */

    /* 
    #[test]
    fn test_option_policy_no_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, OptionPolicy<NoFoldPolicy>> = BBox::new(vec![10u64, 11u64, 12u64], OptionPolicy::Policy(NoFoldPolicy {}));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.len(), 3);
    } 
    */

    #[test]
    fn test_raw_policy_yes_fold_in() {
        let opt_of_pcon: BBox<Option<u64>, NoPolicy> = BBox::new(Some(10u64), NoPolicy {});
        let pcon_of_opt: Option<BBox<u64, NoPolicy>> = opt_of_pcon.fold_in(); 
        assert_eq!(pcon_of_opt.clone().unwrap(), BBox::new(10u64, NoPolicy {}));
        assert_eq!(pcon_of_opt.unwrap().discard_box(), 10u64);

        let res_of_pcon: BBox<Result<u64, ()>, NoPolicy> = BBox::new(Ok(10u64), NoPolicy {});
        let pcon_of_res: Result<BBox<u64, NoPolicy>, ()> = res_of_pcon.fold_in(); 
        assert_eq!(pcon_of_res.clone().unwrap(), BBox::new(10u64, NoPolicy {}));
        assert_eq!(pcon_of_res.unwrap().discard_box(), 10u64);
    }

    //TODO this test fails bc it uses the default impl of RuntimeFoldIn for AnyPolicy,p
    #[test]
    fn test_any_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, NoPolicy>  = BBox::new(vec![10u64, 11u64, 12u64], NoPolicy {} );
        let pcon_of_vec = pcon_of_vec.into_any_policy();
        assert_eq!(true, pcon_of_vec.policy().can_fold_in());
        let vec_of_pcon: Vec<BBox<u64, AnyPolicy>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.len(), 3);
    } 

    #[test]
    #[should_panic]
    fn test_any_policy_no_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, NoFoldPolicy>  = BBox::new(vec![10u64], NoFoldPolicy { });
        let pcon_of_vec = pcon_of_vec.into_any_policy();
        assert_eq!(false, pcon_of_vec.policy().can_fold_in());
        let vec_of_pcon: Vec<BBox<u64, AnyPolicy>> = pcon_of_vec.fold_in(); // correctly panics
    } 

    #[test]
    fn test_ref_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, NoPolicy>  = BBox::new(vec![10u64, 11u64, 12u64], NoPolicy {});
        let pcon_of_vec: BBox<&Vec<u64>, RefPolicy<'_, NoPolicy>> = pcon_of_vec.as_ref();
        let pcon_of_vec: BBox<Vec<u64>, _> = pcon_of_vec.into_ppr(PrivacyPureRegion::new(|v: &Vec<u64>| v.clone()));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.len(), 3);
    } 

    #[test]
    fn test_and_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, PolicyAnd<NoPolicy, NoPolicy>> = BBox::new(vec![10u64, 11u64, 12u64], PolicyAnd::new(NoPolicy {}, NoPolicy {}));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.len(), 3);
    } 

    #[test]
    fn test_or_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, PolicyOr<NoPolicy, NoPolicy>> = BBox::new(vec![10u64, 11u64, 12u64], PolicyOr::new(NoPolicy {}, NoPolicy {}));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.len(), 3);
    } 


    #[test]
    fn test_option_policy_yes_foldin() {
        let pcon_of_vec: BBox<Vec<u64>, OptionPolicy<NoPolicy>> = BBox::new(vec![10u64, 11u64, 12u64], OptionPolicy::Policy(NoPolicy {}));
        assert_eq!(pcon_of_vec.policy().can_fold_in(), true);
        let vec_of_pcon: Vec<BBox<u64, _>> = pcon_of_vec.fold_in(); 
        assert_eq!(vec_of_pcon.len(), 3);
    } 

}