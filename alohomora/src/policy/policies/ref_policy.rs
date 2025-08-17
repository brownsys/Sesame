use crate::bbox::BBox;
use crate::context::UnprotectedContext;
use crate::policy::{NoPolicy, Policy, Reason, Unjoinable};
use std::fmt::{Debug, Formatter};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RefPolicy<'a, P: Policy + ?Sized> {
    policy: &'a P,
}

unsafe impl<'a, P: Policy + ?Sized> Send for RefPolicy<'a, P> {}
unsafe impl<'a, P: Policy + ?Sized> Sync for RefPolicy<'a, P> {}

impl<'a, P: Policy + ?Sized> RefPolicy<'a, P> {
    pub fn new(policy: &'a P) -> Self {
        RefPolicy { policy }
    }
    pub fn policy(&self) -> &'a P {
        self.policy
    }
}

Unjoinable!(RefPolicy<'a, P> where P: Policy + ?Sized);

impl<'a, P: Policy + ?Sized> Policy for RefPolicy<'a, P> {
    fn name(&self) -> String {
        format!("RefPolicy({})", self.policy.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.policy.check(context, reason)
    }
}

// Upcast to a ref object.
impl<'a: 'static, P: Policy + Sized> From<RefPolicy<'a, P>> for RefPolicy<'a, dyn Policy> {
    fn from(value: RefPolicy<'a, P>) -> RefPolicy<'a, dyn Policy> {
        RefPolicy::new(value.policy)
    }
}

// RefPolicy<'_, NoPolicy> can be discarded, logged, etc
impl<'a, T> BBox<T, RefPolicy<'a, NoPolicy>> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}
impl<'a, T: Debug> Debug for BBox<T, RefPolicy<'a, NoPolicy>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BBox").field("data", self.data()).finish()
    }
}
impl<'a, T: PartialEq> PartialEq for BBox<T, RefPolicy<'a, NoPolicy>> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}
