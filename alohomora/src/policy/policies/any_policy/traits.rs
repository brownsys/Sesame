use crate::policy::{
    AnyPolicyDyn, AnyPolicyTrait, Policy, PolicyAnd, PolicyOr, Specializable,
};
use std::any::Any;

// If a policy meets this bound, then it can be placed inside a type-erased AnyPolicy
pub trait AnyPolicyable: Any + Policy + Specializable {}

impl<P: Any + Policy + Specializable> AnyPolicyable for P {}

// You should implement this for combination of traits you can about preserving through
// policies inside a SesameType from_enum and into_enum transformation.
// E.g., Tahini should implement this for Any + Policy + Serialize.
pub trait PolicyDyn: AnyPolicyable + Sync + Send {
    fn upcast_super(&self) -> &dyn AnyPolicyTrait;
    fn upcast_super_mut(&mut self) -> &mut dyn AnyPolicyTrait;
    fn upcast_super_boxed(self: Box<Self>) -> Box<dyn AnyPolicyTrait>;
    fn upcast_ref(&self) -> &dyn Any;
    fn upcast_mut(&mut self) -> &mut dyn Any;
    fn upcast_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_pref(&self) -> &dyn Policy;
    fn upcast_pbox(self: Box<Self>) -> Box<dyn Policy>;
    // To avoid type erasure issues.
    fn can_fold_in_erased(&self) -> bool;
    // default.
    fn no_policy() -> Box<Self>;
    fn and_policy(and: PolicyAnd<AnyPolicyDyn<Self>, AnyPolicyDyn<Self>>) -> AnyPolicyDyn<Self>;
    fn or_policy(or: PolicyOr<AnyPolicyDyn<Self>, AnyPolicyDyn<Self>>) -> AnyPolicyDyn<Self>;
}

// Relates a type P to the corresponding PolicyDyn trait object.
// E.g. relates every P: Any + Policy to dyn PolicyAnyTrait.
pub trait PolicyDynRelation<P>: PolicyDyn
where
    P: AnyPolicyable,
{
    fn boxed_dyn(t: P) -> Box<Self>;
}

// For converting upwards between PolicyDyn (e.g. from AnyPolicySerialize
// or AnyPolicyClone to AnyPolicyTrait).
pub trait PolicyDynInto<PDyn: PolicyDyn + ?Sized> {
    fn policy_dyn_into_ref(&self) -> &PDyn;
    fn policy_dyn_into_boxed(self: Box<Self>) -> Box<PDyn>;
}
impl<PDyn: PolicyDyn + ?Sized> PolicyDynInto<PDyn> for PDyn {
    fn policy_dyn_into_ref(&self) -> &PDyn {
        self
    }
    fn policy_dyn_into_boxed(self: Box<Self>) -> Box<PDyn> {
        self
    }
}

// Marker trait we use for specialization.
pub(crate) trait AnyPolicyMarker<P: PolicyDyn + ?Sized> {
    fn into_any_policy(self) -> Box<P>;
}
impl<P: PolicyDyn + ?Sized, PTarget: PolicyDyn + ?Sized> AnyPolicyMarker<PTarget>
    for AnyPolicyDyn<P>
where
    P: PolicyDynInto<PTarget>,
{
    fn into_any_policy(self) -> Box<PTarget> {
        self.convert_to().into_inner()
    }
}
