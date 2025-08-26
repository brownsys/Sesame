#![feature(negative_impls)]

#[macro_use]
extern crate static_assertions;

use alohomora::context::UnprotectedContext;
use alohomora::policy::Reason;
use alohomora::policy::SimplePolicy;
use alohomora_derive::NoFoldIn;

#[derive(NoFoldIn, Clone, Debug, PartialEq, Eq)]
struct NoFoldPolicy {
    pub attr: String,
}

impl SimplePolicy for NoFoldPolicy {
    fn simple_name(&self) -> String {
        String::from("NoFoldInPolicy")
    }
    fn simple_check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        true
    }
    fn simple_join_direct(&mut self, other: &mut Self) {
        self.attr = format!("{}+{}", self.attr, other.attr);
    }
}

// This correctly fails to compile.
#[test]
fn test_fold_in_denied() {
    use alohomora::fold_in::FoldInAllowed;
    assert_not_impl_any!(NoFoldPolicy: FoldInAllowed);
}
