use crate::context::UnprotectedContext;
use crate::fold_in::RuntimeFoldIn;
use crate::policy::{Policy, Reason};
use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};

// Any (owned) Policy.
pub(crate) trait TypeIdPolicyTrait: Policy + Any {
    fn clone_boxed(&self) -> Box<dyn TypeIdPolicyTrait>;
    fn can_fold_in_boxed(&self) -> bool;
}
impl<P: Policy + Clone + 'static> TypeIdPolicyTrait for P {
    fn clone_boxed(&self) -> Box<dyn TypeIdPolicyTrait> {
        Box::new(self.clone())
    }
    fn can_fold_in_boxed(&self) -> bool {
        self.can_fold_in()
    }
}

pub struct AnyPolicy {
    policy: Box<dyn TypeIdPolicyTrait>,
}
impl AnyPolicy {
    pub fn new<P: Policy + Clone + 'static>(p: P) -> Self {
        if TypeId::of::<AnyPolicy>() == TypeId::of::<P>() {
            p.into_any()
        } else {
            Self {
                policy: Box::new(p),
            }
        }
    }
    pub fn is<P: Policy + 'static>(&self) -> bool {
        TypeId::of::<P>() == self.policy.as_ref().type_id()
    }
    pub fn specialize<P: Policy + 'static>(self) -> Result<P, String> {
        if TypeId::of::<AnyPolicy>() == TypeId::of::<P>() {
            let b = Box::new(self);
            let raw = Box::into_raw(b);
            let raw = raw as *mut P;
            Ok(*unsafe { Box::from_raw(raw) })
        } else if self.is::<P>() {
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

    pub(crate) fn policy(&self) -> &dyn TypeIdPolicyTrait {
        self.policy.as_ref()
    }
}

impl Policy for AnyPolicy {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.policy.check(context, reason)
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        self.policy.join(other)
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        self.policy.join(other)
    }
    fn into_any(self) -> AnyPolicy
    where
        Self: Sized,
    {
        self
    }
}
impl Clone for AnyPolicy {
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone_boxed(),
        }
    }
}
impl Debug for AnyPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

impl PartialEq for AnyPolicy {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
