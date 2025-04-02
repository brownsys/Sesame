use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use erased_serde::serialize_trait_object;

use crate::context::UnprotectedContext;
use crate::policy::{NoPolicy, Policy, Reason};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::AnyPolicy;

// Any (owned) Policy.
trait TypeIdSerPolicyTrait: Policy + Any + erased_serde::Serialize{
    fn clone(&self) -> Box<dyn TypeIdSerPolicyTrait>;
}
impl<P: Policy + Clone + Serialize + DeserializeOwned + 'static> TypeIdSerPolicyTrait for P {
    fn clone(&self) -> Box<dyn TypeIdSerPolicyTrait> {
        Box::new(self.clone())
    }
}

serialize_trait_object!(TypeIdSerPolicyTrait);

pub struct TahiniPolicy {
    policy: Box<dyn TypeIdSerPolicyTrait>,
}

impl serde::Serialize for TahiniPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
                erased_serde::serialize(&(*(self.policy)), serializer)
        
    }
}

impl TahiniPolicy {
    pub fn new<P: Policy + Clone + Serialize + DeserializeOwned + 'static>(p: P) -> Self {
        // if TypeId::of::<TahiniPolicy>() == TypeId::of::<P>() {
        //     p.clone()
        // } else {
            Self {
                policy: Box::new(p),
            }
        // }
    }
    pub fn is<P: Policy + 'static>(&self) -> bool {
        TypeId::of::<P>() == self.policy.as_ref().type_id()
    }
    pub fn specialize<P: Policy + 'static>(self) -> Result<P, String> {
        if TypeId::of::<TahiniPolicy>() == TypeId::of::<P>() {
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
}

impl Policy for TahiniPolicy {
    fn name(&self) -> String {
        format!("TahiniPolicy({})", self.policy.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.policy.check(context, reason)
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        //TODO(douk): Add to policies an optional tahini_join or smth like that
        todo!()
        // self.policy.join(other)
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        //TODO(douk): Add to policies an optional tahini_join or smth like that
        todo!()
        // self.policy.join(other)
    }
    fn into_any(self) -> AnyPolicy where Self: Sized {
        AnyPolicy::new(self)
    }
}
impl Clone for TahiniPolicy {
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone()
        }
    }
}
impl Debug for TahiniPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

impl PartialEq for TahiniPolicy {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}
