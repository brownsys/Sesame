use crate::policy::policies::any_policy::traits::AnyPolicyMarker;
use crate::policy::{
    AnyPolicy, AnyPolicyDyn, AnyPolicyable, NoPolicy, Policy, PolicyAnd, PolicyDyn, PolicyDynInto,
    PolicyDynRelation, PolicyOr,
};
use dyn_clone::DynClone;
use std::any::Any;

// AnyPolicy with Clone.
pub trait AnyPolicyCloneDyn: AnyPolicyable + DynClone {
    // These upcasts would be unneeded with trait object upcasting, but we are not using a new
    // enough Rust version :(
    fn upcast_any_policy(&self) -> &dyn AnyPolicyDyn;
    fn upcast_any_policy_mut(&mut self) -> &mut dyn AnyPolicyDyn;
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyDyn>;
}
impl<P: AnyPolicyable + DynClone> AnyPolicyCloneDyn for P {
    fn upcast_any_policy(&self) -> &dyn AnyPolicyDyn {
        self
    }
    fn upcast_any_policy_mut(&mut self) -> &mut dyn AnyPolicyDyn {
        self
    }
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyDyn> {
        self
    }
}
impl PolicyDyn for dyn AnyPolicyCloneDyn {
    fn upcast_super(&self) -> &dyn AnyPolicyDyn {
        self.upcast_any_policy()
    }
    fn upcast_super_mut(&mut self) -> &mut dyn AnyPolicyDyn {
        self.upcast_any_policy_mut()
    }
    fn upcast_super_boxed(self: Box<Self>) -> Box<dyn AnyPolicyDyn> {
        self.upcast_any_policy_box()
    }
    fn upcast_ref(&self) -> &dyn Any {
        self.upcast_any_policy().upcast_any()
    }
    fn upcast_mut(&mut self) -> &mut dyn Any {
        self.upcast_any_policy_mut().upcast_any_mut()
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self.upcast_any_policy_box().upcast_any_box()
    }
    fn upcast_pref(&self) -> &dyn Policy {
        self.upcast_any_policy().upcast_policy()
    }
    fn upcast_pbox(self: Box<Self>) -> Box<dyn Policy> {
        self.upcast_any_policy_box().upcast_policy_box()
    }
    fn can_fold_in_erased(&self) -> bool {
        self.upcast_any_policy().can_fold_in_erased()
    }
    fn no_policy() -> Box<Self> {
        Self::boxed_dyn(NoPolicy {})
    }
    fn and_policy(and: PolicyAnd<AnyPolicy<Self>, AnyPolicy<Self>>) -> AnyPolicy<Self> {
        AnyPolicy::new(and)
    }
    fn or_policy(or: PolicyOr<AnyPolicy<Self>, AnyPolicy<Self>>) -> AnyPolicy<Self> {
        AnyPolicy::new(or)
    }
}
impl<P: AnyPolicyable + DynClone> PolicyDynRelation<P> for dyn AnyPolicyCloneDyn {
    default fn boxed_dyn(t: P) -> Box<Self> {
        Box::new(t)
    }
}
impl<P: AnyPolicyable + DynClone + AnyPolicyMarker<dyn AnyPolicyCloneDyn>> PolicyDynRelation<P>
    for dyn AnyPolicyCloneDyn
{
    fn boxed_dyn(t: P) -> Box<Self> {
        t.into_any_policy()
    }
}

// Convert to AnyPolicyTrait.
impl PolicyDynInto<dyn AnyPolicyDyn> for dyn AnyPolicyCloneDyn {
    fn policy_dyn_into_ref(&self) -> &dyn AnyPolicyDyn {
        self.upcast_any_policy()
    }
    fn policy_dyn_into_boxed(self: Box<Self>) -> Box<dyn AnyPolicyDyn> {
        self.upcast_any_policy_box()
    }
}

mod __validation {
    #[allow(dead_code)]
    fn example<P: super::AnyPolicyCloneDyn>(_p: P) {}
    #[allow(dead_code)]
    fn test_me(p: super::AnyPolicy<dyn super::AnyPolicyCloneDyn>) {
        example(p)
    }
}
// End basic AnyPolicy with Clone impls.
