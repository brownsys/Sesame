#![feature(negative_impls)]

#[macro_use]
extern crate static_assertions;

use alohomora::policy::{Policy, AnyPolicy};
use alohomora_derive::NoFoldIn;
use alohomora::bbox::BBox;
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

// This correctly fails to compile. 
#[test]
fn test_fold_in_denied() {
    use alohomora::fold_in::FoldInAllowed;
    assert_not_impl_any!(NoFoldPolicy: FoldInAllowed);
}
