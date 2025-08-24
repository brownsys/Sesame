use crate::policy::policies::any_policy::traits::AnyPolicyMarker;
use crate::policy::{
    AnyPolicyDyn, AnyPolicyTrait, AnyPolicyable, NoPolicy, Policy, PolicyAnd,
    PolicyDyn, PolicyDynInto, PolicyDynRelation, PolicyOr,
};
use dyn_clone::DynClone;
use std::any::Any;

// AnyPolicy with Clone.
pub trait AnyPolicyClone: AnyPolicyable + DynClone {
    // These upcasts would be unneeded with trait object upcasting, but we are not using a new
    // enough Rust version :(
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait;
    fn upcast_any_policy_mut(&mut self) -> &mut dyn AnyPolicyTrait;
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait>;
}
impl<P: AnyPolicyable + DynClone> AnyPolicyClone for P {
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait {
        self
    }
    fn upcast_any_policy_mut(&mut self) -> &mut dyn AnyPolicyTrait {
        self
    }
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait> {
        self
    }
}
impl PolicyDyn for dyn AnyPolicyClone {
    fn upcast_super(&self) -> &dyn AnyPolicyTrait {
        self.upcast_any_policy()
    }
    fn upcast_super_mut(&mut self) -> &mut dyn AnyPolicyTrait {
        self.upcast_any_policy_mut()
    }
    fn upcast_super_boxed(self: Box<Self>) -> Box<dyn AnyPolicyTrait> {
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
    fn and_policy(and: PolicyAnd<AnyPolicyDyn<Self>, AnyPolicyDyn<Self>>) -> AnyPolicyDyn<Self> {
        AnyPolicyDyn::new(and)
    }
    fn or_policy(or: PolicyOr<AnyPolicyDyn<Self>, AnyPolicyDyn<Self>>) -> AnyPolicyDyn<Self> {
        AnyPolicyDyn::new(or)
    }
}
impl<P: AnyPolicyable + DynClone> PolicyDynRelation<P> for dyn AnyPolicyClone {
    default fn boxed_dyn(t: P) -> Box<Self> {
        Box::new(t)
    }
}
impl<P: AnyPolicyable + DynClone + AnyPolicyMarker<dyn AnyPolicyClone>> PolicyDynRelation<P>
    for dyn AnyPolicyClone
{
    fn boxed_dyn(t: P) -> Box<Self> {
        t.into_any_policy()
    }
}

// Convert to AnyPolicyTrait.
impl PolicyDynInto<dyn AnyPolicyTrait> for dyn AnyPolicyClone {
    fn policy_dyn_into_ref(&self) -> &dyn AnyPolicyTrait {
        self.upcast_any_policy()
    }
    fn policy_dyn_into_boxed(self: Box<Self>) -> Box<dyn AnyPolicyTrait> {
        self.upcast_any_policy_box()
    }
}

mod __validation {
    #[allow(dead_code)]
    fn example<P: super::AnyPolicyClone>(_p: P) {}
    #[allow(dead_code)]
    fn test_me(p: super::AnyPolicyDyn<dyn super::AnyPolicyClone>) {
        example(p)
    }
}
// End basic AnyPolicy with Clone impls.
