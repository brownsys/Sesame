use crate::policy::policies::any_policy::traits::AnyPolicyMarker;
use crate::policy::{
    AnyPolicyDyn, AnyPolicyTrait, AnyPolicyable, NoPolicy, Policy, PolicyAnd, PolicyDyn,
    PolicyDynInto, PolicyDynRelation, PolicyOr,
};
use erased_serde::{serialize_trait_object, Serialize};
use std::any::Any;

// Example: Now we can preserve Any + Policy + Serialize through SesameType transformations.
// This part should be macro-ed for custom combinations of Any + Policy + <traits>.
pub trait AnyPolicySerialize: AnyPolicyable + Serialize {
    // These upcasts would be unneeded with trait object upcasting but we are not using a new
    // enough Rust version :(
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait;
    fn upcast_any_policy_mut(&mut self) -> &mut dyn AnyPolicyTrait;
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait>;
    fn upcast_serialize(&self) -> &dyn Serialize;
    fn upcast_serialize_box(self: Box<Self>) -> Box<dyn Serialize>;
}
impl<P: AnyPolicyable + Serialize> AnyPolicySerialize for P {
    fn upcast_any_policy(&self) -> &dyn AnyPolicyTrait {
        self
    }
    fn upcast_any_policy_mut(&mut self) -> &mut dyn AnyPolicyTrait {
        self
    }
    fn upcast_any_policy_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait> {
        self
    }
    fn upcast_serialize(&self) -> &dyn Serialize {
        self
    }
    fn upcast_serialize_box(self: Box<Self>) -> Box<dyn Serialize> {
        self
    }
}
impl PolicyDyn for dyn AnyPolicySerialize {
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
impl<P: AnyPolicyable + Serialize> PolicyDynRelation<P> for dyn AnyPolicySerialize {
    default fn boxed_dyn(t: P) -> Box<Self> {
        Box::new(t)
    }
}
impl<P: AnyPolicyable + Serialize + AnyPolicyMarker<dyn AnyPolicySerialize>> PolicyDynRelation<P>
    for dyn AnyPolicySerialize
{
    fn boxed_dyn(t: P) -> Box<Self> {
        t.into_any_policy()
    }
}
serialize_trait_object!(AnyPolicySerialize);

// Convert to AnyPolicyTrait.
impl PolicyDynInto<dyn AnyPolicyTrait> for dyn AnyPolicySerialize {
    fn policy_dyn_into_ref(&self) -> &dyn AnyPolicyTrait {
        self.upcast_any_policy()
    }
    fn policy_dyn_into_boxed(self: Box<Self>) -> Box<dyn AnyPolicyTrait> {
        self.upcast_any_policy_box()
    }
}

mod __validation {
    #[allow(dead_code)]
    fn example<P: super::AnyPolicySerialize>(_p: P) {}
    #[allow(dead_code)]
    fn test_me(p: super::AnyPolicyDyn<dyn super::AnyPolicySerialize>) {
        example(p)
    }
}
// End of Macro.
