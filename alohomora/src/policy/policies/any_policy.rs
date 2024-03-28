use std::any::{Any, TypeId};
use crate::context::UnprotectedContext;
use crate::policy::Policy;

// Any (owned) Policy.
trait TypeIdPolicyTrait: Policy + Any {
    fn clone(&self) -> Box<dyn TypeIdPolicyTrait>;
}
impl<P: Policy + Clone + 'static> TypeIdPolicyTrait for P {
    fn clone(&self) -> Box<dyn TypeIdPolicyTrait> {
        Box::new(self.clone())
    }
}

pub struct AnyPolicy {
    policy: Box<dyn TypeIdPolicyTrait>,
}
impl AnyPolicy {
    pub fn new<P: Policy + Clone + 'static>(p: P) -> Self {
        Self {
            policy: Box::new(p),
        }
    }
    pub fn is<P: Policy + 'static>(&self) -> bool {
        TypeId::of::<P>() == self.policy.as_ref().type_id()
    }
    pub fn specialize<P: Policy + 'static>(self) -> Result<P, String> {
        if self.is::<P>() {
            let raw = Box::into_raw(self.policy);
            let raw = raw as *mut P;
            Ok(*unsafe { Box::from_raw(raw) })
        } else {
            Err(format!(
                "Cannot convert '{}' to '{:?}'",
                self.name(),
                TypeId::of::<P>()
            ))
        }
    }
}
impl Policy for AnyPolicy {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.name())
    }
    fn check(&self, context: &UnprotectedContext) -> bool {
        self.policy.check(context)
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        self.policy.join(other)
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        self.policy.join(other)
    }
}
impl Clone for AnyPolicy {
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone()
        }
    }
}