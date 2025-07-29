use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyBB, AnyPolicyTrait, Policy, Reason};
use std::any::TypeId;
use dyn_clone::DynClone;
use crate::policy::{AnyPolicyable, PolicyDyn, PolicyDynRelation};

// Type-erased AnyPolicy that can pose dyn trait object obligations
// on the policy inside of it.
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

// AnyPolicyDyn is a Policy.
impl<P: PolicyDyn + ?Sized> Policy for AnyPolicyDyn<P> {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.upcast_pref().name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.policy.upcast_pref().check(context, reason)
    }
    fn join(&self, other: AnyPolicyBB) -> Result<AnyPolicyBB, ()> {
        self.policy.upcast_pref().join(other)
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> {
        // we can implement this by making AndPolicy<P1, P2> satisfy any PolicyDynRelation
        // both elements satisfy.
        // Then, we can do join here and do some type-erasure to get a Self.
        todo!()
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