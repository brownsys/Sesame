use std::any::Any;
use crate::fold_in::RuntimeFoldIn;
use crate::policy::{AnyPolicyDyn, AnyPolicyable, NoPolicy, Policy, PolicyAnd, PolicyDyn, PolicyDynRelation, PolicyOr};
use crate::policy::policies::any_policy::AnyPolicyMarker;

// Basic trait setup for AnyPolicy without any additional obligations on the dyn type.
pub trait AnyPolicyTrait : AnyPolicyable {
    // These upcasts would be unneeded with trait object upcasting, but we are not using a new
    // enough Rust version :(
    fn upcast_any(&self) -> &dyn Any;
    fn upcast_any_mut(&mut self) -> &mut dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_policy(&self) -> &dyn Policy;
    fn upcast_policy_box(self: Box<Self>) -> Box<dyn Policy>;
    fn can_fold_in_erased(&self) -> bool;
}
impl<P: AnyPolicyable> AnyPolicyTrait for P {
    fn upcast_any(&self) -> &dyn Any { self }
    fn upcast_any_mut(&mut self) -> &mut dyn Any { self }
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any> { self }
    fn upcast_policy(&self) -> &dyn Policy { self }
    fn upcast_policy_box(self: Box<Self>) -> Box<dyn Policy> { self }
    fn can_fold_in_erased(&self) -> bool {
        self.can_fold_in()
    }
}
impl PolicyDyn for dyn AnyPolicyTrait {
    fn upcast_super(&self) -> &dyn AnyPolicyTrait { self }
    fn upcast_super_mut(&mut self) -> &mut dyn AnyPolicyTrait { self }
    fn upcast_super_boxed(self: Box<Self>) -> Box<dyn AnyPolicyTrait> { self }
    fn upcast_ref(&self) -> &dyn Any { self.upcast_any() }
    fn upcast_mut(&mut self) -> &mut dyn Any { self.upcast_any_mut() }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> { self.upcast_any_box() }
    fn upcast_pref(&self) -> &dyn Policy { self.upcast_policy() }
    fn upcast_pbox(self: Box<Self>) -> Box<dyn Policy> { self.upcast_policy_box() }
    fn can_fold_in_erased(&self) -> bool { self.can_fold_in_erased() }
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
impl<P: AnyPolicyable> PolicyDynRelation<P> for dyn AnyPolicyTrait {
    default fn boxed_dyn(t: P) -> Box<Self> { Box::new(t) }
}
impl<P: AnyPolicyable + AnyPolicyMarker<dyn AnyPolicyTrait>> PolicyDynRelation<P> for dyn AnyPolicyTrait {
    fn boxed_dyn(t: P) -> Box<Self> { t.into_any_policy() }
}

mod __validation {
    pub fn example<P: super::AnyPolicyTrait>(_p: P) {}
    pub fn test_me(p: super::AnyPolicyDyn<dyn super::AnyPolicyTrait>) {
        example(p)
    }
}
// End basic AnyPolicy impls.