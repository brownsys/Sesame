use crate::context::UnprotectedContext;
use crate::pcon::PCon;
use crate::policy::{Policy, Reason};
use serde::Serialize;
use std::fmt::{Debug, Formatter};

// NoPolicy can be directly discarded.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct NoPolicy {}

impl NoPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Policy for NoPolicy {
    fn name(&self) -> String {
        String::from("NoPolicy")
    }
    fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        true
    }
}

impl Default for NoPolicy {
    fn default() -> Self {
        NoPolicy {}
    }
}

// NoPolicy can be discarded, logged, etc
impl<T> PCon<T, NoPolicy> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}
impl<T: Debug> Debug for PCon<T, NoPolicy> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PCon")
            .field("data", self.data())
            .field("policy", &"NoPolicy")
            .finish()
    }
}
impl<T: PartialEq> PartialEq for PCon<T, NoPolicy> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}
