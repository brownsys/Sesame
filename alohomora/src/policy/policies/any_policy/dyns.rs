use std::any::Any;
use dyn_clone::DynClone;
use erased_serde::{serialize_trait_object, Serialize};
use crate::fold_in::RuntimeFoldIn;
use crate::policy::{AnyPolicyDyn, NoPolicy, Policy, PolicyAnd};
use crate::policy::policies::any_policy::r#type::AnyPolicyMarker;

// If a policy meets this bound, then it can be placed inside a type-erased AnyPolicy
pub trait AnyPolicyable : Any + Policy {}
impl <P: Any + Policy> AnyPolicyable for P {}

// You should implement this for combination of traits you can about preserving through
// policies inside a SesameType from_enum and into_enum transformation.
// E.g., Tahini should implement this for Any + Policy + Serialize.
pub trait PolicyDyn: AnyPolicyable + Sync + Send {
    fn upcast_super(self: Box<Self>) -> Box<dyn AnyPolicyTrait>;
    fn upcast_ref(&self) -> &dyn Any;
    fn upcast_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_pref(&self) -> &dyn Policy;
    fn upcast_pbox(self: Box<Self>) -> Box<dyn Policy>;
    // To avoid type erasure issues.
    fn can_fold_in_erased(&self) -> bool;
    // default.
    fn no_policy() -> Box<Self>;
    fn and_policy(and: PolicyAnd<AnyPolicyDyn<Self>, AnyPolicyDyn<Self>>) -> AnyPolicyDyn<Self>;
}

// Relates a type P to the corresponding PolicyDyn trait object.
// E.g. relates every P: Any + Policy to dyn PolicyAnyTrait.
pub trait PolicyDynRelation<P> : PolicyDyn where P: AnyPolicyable {
    fn boxed_dyn(t: P) -> Box<Self>;
}

// Basic trait setup for AnyPolicy without any additional obligations on the dyn type.
pub trait AnyPolicyTrait : AnyPolicyable {
    // These upcasts would be unneeded with trait object upcasting, but we are not using a new
    // enough Rust version :(
    fn upcast_any(&self) -> &dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_policy(&self) -> &dyn Policy;
    fn upcast_policy_box(self: Box<Self>) -> Box<dyn Policy>;
    fn can_fold_in_erased(&self) -> bool;
}
impl<P: AnyPolicyable> AnyPolicyTrait for P {
    fn upcast_any(&self) -> &dyn Any { self }
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any> { self }
    fn upcast_policy(&self) -> &dyn Policy { self }
    fn upcast_policy_box(self: Box<Self>) -> Box<dyn Policy> { self }
    fn can_fold_in_erased(&self) -> bool {
        self.can_fold_in()
    }
}
impl PolicyDyn for dyn AnyPolicyTrait {
    fn upcast_super(self: Box<Self>) -> Box<dyn AnyPolicyTrait> { self }
    fn upcast_ref(&self) -> &dyn Any { self.upcast_any() }
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
}
impl<P: AnyPolicyable> PolicyDynRelation<P> for dyn AnyPolicyTrait {
    default fn boxed_dyn(t: P) -> Box<Self> { Box::new(t) }
}
impl<P: AnyPolicyable + AnyPolicyMarker<dyn AnyPolicyTrait>> PolicyDynRelation<P> for dyn AnyPolicyTrait {
    fn boxed_dyn(t: P) -> Box<Self> { t.into_inner() }
}
// End basic AnyPolicy impls.

// AnyPolicy with Clone.
pub trait AnyPolicyClone : AnyPolicyable + DynClone {
    // These upcasts would be unneeded with trait object upcasting, but we are not using a new
    // enough Rust version :(
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait;
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait>;
}
impl<P: AnyPolicyable + DynClone> AnyPolicyClone for P {
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait { self }
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait> { self }
}
impl PolicyDyn for dyn AnyPolicyClone {
    fn upcast_super(self: Box<Self>) -> Box<dyn AnyPolicyTrait> { self.upcast_any_policy_box() }
    fn upcast_ref(&self) -> &dyn Any { self.upcast_any_policy().upcast_any() }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> { self.upcast_any_policy_box().upcast_any_box() }
    fn upcast_pref(&self) -> &dyn Policy { self.upcast_any_policy().upcast_policy() }
    fn upcast_pbox(self: Box<Self>) -> Box<dyn Policy> { self.upcast_any_policy_box().upcast_policy_box() }
    fn can_fold_in_erased(&self) -> bool { self.upcast_any_policy().can_fold_in_erased() }
    fn no_policy() -> Box<Self> {
        Self::boxed_dyn(NoPolicy {})
    }
    fn and_policy(and: PolicyAnd<AnyPolicyDyn<Self>, AnyPolicyDyn<Self>>) -> AnyPolicyDyn<Self> {
        AnyPolicyDyn::new(and)
    }
}
impl<P: AnyPolicyable + DynClone> PolicyDynRelation<P> for dyn AnyPolicyClone {
    default fn boxed_dyn(t: P) -> Box<Self> { Box::new(t) }
}
impl<P: AnyPolicyable + DynClone + AnyPolicyMarker<dyn AnyPolicyClone>> PolicyDynRelation<P> for dyn AnyPolicyClone {
    fn boxed_dyn(t: P) -> Box<Self> { t.into_inner() }
}
// End basic AnyPolicy with Clone impls.

// Example: Now we can preserve Any + Policy + Serialize through SesameType transformations.
// This part should be macro-ed for custom combinations of Any + Policy + <traits>.
pub trait AnyPolicySerialize: AnyPolicyTrait + Serialize {
    // These upcasts would be unneeded with trait object upcasting but we are not using a new
    // enough Rust version :(
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait;
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait>;
    fn upcast_serialize(&self) -> &dyn Serialize;
    fn upcast_serialize_box(self: Box<Self>) -> Box<dyn Serialize>;
}
impl<P: AnyPolicyTrait + Serialize> AnyPolicySerialize for P {
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait { self }
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait> { self }
    fn upcast_serialize(&self) -> &dyn Serialize {
        self
    }
    fn upcast_serialize_box(self: Box<Self>) -> Box<dyn Serialize> { self }
}
impl PolicyDyn for dyn AnyPolicySerialize {
    fn upcast_super(self: Box<Self>) -> Box<dyn AnyPolicyTrait> { self.upcast_any_policy_box()}
    fn upcast_ref(&self) -> &dyn Any {
        self.upcast_any_policy().upcast_any()
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self.upcast_any_policy_box().upcast_any_box()
    }
    fn upcast_pref(&self) -> &dyn Policy { self.upcast_any_policy().upcast_policy() }
    fn upcast_pbox(self: Box<Self>) -> Box<dyn Policy> { self.upcast_any_policy_box().upcast_policy_box() }
    fn can_fold_in_erased(&self) -> bool { self.upcast_any_policy().can_fold_in_erased() }
    fn no_policy() -> Box<Self> {
        Self::boxed_dyn(NoPolicy {})
    }
    fn and_policy(and: PolicyAnd<AnyPolicyDyn<Self>, AnyPolicyDyn<Self>>) -> AnyPolicyDyn<Self> {
        AnyPolicyDyn::new(and)
    }
}
impl<P: AnyPolicyTrait + Serialize> PolicyDynRelation<P> for dyn AnyPolicySerialize {
    default fn boxed_dyn(t: P) -> Box<Self> {
        Box::new(t)
    }
}
impl<P: AnyPolicyTrait + Serialize + AnyPolicyMarker<dyn AnyPolicySerialize>> PolicyDynRelation<P> for dyn AnyPolicySerialize {
    fn boxed_dyn(t: P) -> Box<Self> { t.into_inner() }
}
serialize_trait_object!(AnyPolicySerialize);
// End of Macro.
