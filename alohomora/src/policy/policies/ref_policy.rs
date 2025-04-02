use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicy, Policy, Reason};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
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

impl<'a, P: Policy + ?Sized> Policy for RefPolicy<'a, P> {
    fn name(&self) -> String {
        format!("RefPolicy({})", self.policy.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.policy.check(context, reason)
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> {
        todo!()
    }
}

// Upcast to a ref object.
impl<'a: 'static, P: Policy + Sized> From<RefPolicy<'a, P>> for RefPolicy<'a, dyn Policy> {
    fn from(value: RefPolicy<'a, P>) -> RefPolicy<'a, dyn Policy> {
        RefPolicy::new(value.policy)
    }
}

