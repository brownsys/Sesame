#![feature(negative_impls)]
use alohomora::policy::{Policy, AnyPolicy};
use alohomora_derive::NoFoldIn;
use alohomora::bbox::BBox; 
use alohomora::fold_in::FoldIn; 
use alohomora::context::UnprotectedContext;
use alohomora::policy::Reason; 


#[derive(NoFoldIn, Clone, Debug, PartialEq, Eq)]
struct NoFoldPolicy {
    pub attr: String,
}

impl Policy for NoFoldPolicy {
    fn name(&self) -> String {
        String::from("NoFoldInPolicy")
    }
    fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        true
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        Ok(AnyPolicy::new(self.clone()))
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> {
        Ok(NoFoldPolicy { attr: String::from("") })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FoldPolicy {
    pub attr: String,
}

impl Policy for FoldPolicy {
    fn name(&self) -> String {
        String::from("FoldPolicy")
    }
    fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        true
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        Ok(AnyPolicy::new(self.clone()))
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> {
        Ok(FoldPolicy { attr: String::from("") })
    }
}

#[test]
 fn test_fold_in_ok() {
    let pcon_of_vec: BBox<Vec<u64>, FoldPolicy>  = BBox::new(vec![10u64], FoldPolicy { attr: "".to_string() });
    let vec_of_pcon: Vec<BBox<u64, FoldPolicy>> = pcon_of_vec.fold_in(); 
    assert_eq!(vec_of_pcon.first().unwrap(), &BBox::new(10u64, FoldPolicy { attr: "".to_string() }));
} 

// This correctly fails to compile. 
/*
#[test]
 fn test_fold_in_denied() {
    let pcon_of_vec: BBox<Vec<u64>, NoFoldPolicy>  = BBox::new(vec![10u64], NoFoldPolicy { attr: "".to_string() });
    let vec_of_pcon: Vec<BBox<u64, NoFoldPolicy>> = pcon_of_vec.fold_in(); 
    assert_eq!(vec_of_pcon.first().unwrap(), &BBox::new(10u64, NoFoldPolicy { attr: "".to_string() }));
} 
*/