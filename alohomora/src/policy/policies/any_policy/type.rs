use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyTrait, Direction, Joinable, Policy, Reason};
use std::any::TypeId;
use dyn_clone::DynClone;
use serde::Serialize;
use crate::policy::{AnyPolicyable, PolicyDyn, PolicyDynRelation};

// Marker trait we use for specialization.
pub(super) trait AnyPolicyMarker<P: PolicyDyn + ?Sized> {
    fn into_inner(self) -> Box<P>;
}
impl<P: PolicyDyn + ?Sized> AnyPolicyMarker<P> for AnyPolicyDyn<P> {
    fn into_inner(self) -> Box<P> {
        self.into_inner()
    }
}

// Type-erased AnyPolicy that can pose dyn trait object obligations
// on the policy inside of it.
#[derive(Serialize)]
pub struct AnyPolicyDyn<P: PolicyDyn + ?Sized> {
    policy: Box<P>,
}
impl<P: PolicyDyn + ?Sized> AnyPolicyDyn<P> {
    pub fn new<T: AnyPolicyable>(p: T) -> Self
    where P: PolicyDynRelation<T> {
        Self { policy: P::boxed_dyn(p) }
    }

    pub fn is<T: AnyPolicyable>(&self) -> bool
    where P: PolicyDynRelation<T> {
        TypeId::of::<T>() == self.policy.upcast_ref().type_id()
    }

    pub fn specialize<T: AnyPolicyable>(self) -> Result<T, String>
    where P: PolicyDynRelation<T> {
        if self.is::<T>() {
            Ok(*self.policy.upcast_box().downcast::<T>().unwrap())
        } else {
            Err(format!(
                "Cannot convert '{}' to '{}'",
                self.name(),
                std::any::type_name::<T>(),
            ))
        }
    }
    pub fn specialize_ref<T: AnyPolicyable>(&self) -> Result<&T, String>
    where P: PolicyDynRelation<T> {
        if self.is::<T>() {
            Ok(self.policy.upcast_ref().downcast_ref().unwrap())
        } else {
            Err(format!(
                "Cannot convert '{}' to '{}'",
                self.name(),
                std::any::type_name::<T>(),
            ))
        }
    }

    pub fn inner(&self) -> &P {
        self.policy.as_ref()
    }
    pub fn into_inner(self) -> Box<P> {
        self.policy
    }
}

// Joining AnyPolicyDyns.
// TODO(babman): make any policy joinable.
impl<P: PolicyDyn + ?Sized> Joinable for AnyPolicyDyn<P> {
    fn direction_to<P2: AnyPolicyable>(&self, p: &P2) -> Direction
    where
        Self: Sized,
    {
        todo!()
    }
    fn join_in<P2: AnyPolicyable>(&mut self, p: &mut P2, direction: Direction) -> bool
    where
        Self: Sized,
    {
        todo!()
    }
    fn join_direct(&mut self, p: &mut Self) -> bool
    where
        Self: Sized,
    {
        todo!()
    }
}

// AnyPolicyDyn is a Policy.
impl<P: PolicyDyn + ?Sized> Policy for AnyPolicyDyn<P> {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.upcast_pref().name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.policy.upcast_pref().check(context, reason)
    }
}

// AnyPolicyDyn is Clone if it obligates trait object to be Clone as well.
impl<P: PolicyDyn + DynClone + ?Sized> Clone for AnyPolicyDyn<P> {
    fn clone(&self) -> Self {
        Self { policy: dyn_clone::clone_box(self.policy.as_ref()) }
    }
}

// Convert to parent.
impl <P: PolicyDyn + ?Sized> AnyPolicyDyn<P> {
    pub fn upcast_super_box(self) -> AnyPolicyDyn<dyn AnyPolicyTrait> {
        let policy = self.policy;
        AnyPolicyDyn { policy: policy.upcast_super() }
    }
}

impl <P: PolicyDyn + ?Sized> Default for AnyPolicyDyn<P> {
    fn default() -> Self {
        Self { policy: P::no_policy() }
    }
}