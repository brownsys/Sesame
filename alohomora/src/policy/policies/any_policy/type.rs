use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyClone, AnyPolicySerialize, AnyPolicyTrait, Policy, PolicyAnd, PolicyDynInto, PolicyOr, Reason, Specializable, SpecializationEnum, Specialize};
use std::any::{Any, TypeId};
use std::ops::Deref;
use dyn_clone::DynClone;
use serde::Serialize;
use crate::policy::{AnyPolicyable, PolicyDyn, PolicyDynRelation};

// Aliases for ease of use.
pub type AnyPolicyCC = AnyPolicyDyn<dyn AnyPolicyClone>;
pub type AnyPolicyBB = AnyPolicyDyn<dyn AnyPolicyTrait>;
pub type AnyPolicySS = AnyPolicyDyn<dyn AnyPolicySerialize>;

// Type-erased AnyPolicy that can pose dyn trait object obligations
// on the policy inside of it.
#[derive(Serialize)]
pub struct AnyPolicyDyn<P: PolicyDyn + ?Sized> {
    policy: Box<P>,
}
impl<P: PolicyDyn + ?Sized> AnyPolicyDyn<P> {
    // Constructor.
    pub fn new<T: AnyPolicyable>(p: T) -> Self
    where P: PolicyDynRelation<T> {
        Self { policy: P::boxed_dyn(p) }
    }

    // Check what the concrete type is.
    pub fn is<T: AnyPolicyable>(&self) -> bool
    where P: PolicyDynRelation<T> {
        TypeId::of::<T>() == self.policy.upcast_ref().type_id()
    }

    // Specialize the type down to the concrete type (down cast).
    pub fn specialize_top<T: AnyPolicyable>(self) -> Result<T, String>
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
    pub fn specialize_top_ref<T: AnyPolicyable>(&self) -> Result<&T, String>
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

    // Accessing internal policy object.
    pub fn inner(&self) -> &P {
        self.policy.as_ref()
    }
    pub fn into_inner(self) -> Box<P> {
        self.policy
    }

    // Convert between dyns.
    pub fn convert_to<PDyn: PolicyDyn + ?Sized>(self) -> AnyPolicyDyn<PDyn>
    where P: PolicyDynInto<PDyn> {
        AnyPolicyDyn { policy: self.policy.policy_dyn_into_boxed() }
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
    /*
    fn policy_type_enum(&mut self) -> PolicyTypeEnum<'_> {
        PolicyTypeEnum::AnyPolicy(Box::new(self.policy.policy_type_enum()))
    }
    fn can_join_with(&mut self, p: &PolicyTypeEnum<'_>) -> bool {
        self.policy.can_join_with(p)
    }
    fn join(&mut self, p: PolicyTypeEnum<'_>) -> bool {
        self.policy.join(p)
    }
     */
}

// Can specialize any kind of AnyPolicy.
impl<PDyn: PolicyDyn + ?Sized> Specializable for AnyPolicyDyn<PDyn> {
    fn to_specialization_enum(self) -> SpecializationEnum {
        let e = self.policy.to_specialization_enum_box();
        SpecializationEnum::AnyPolicy(Box::new(e))
    }
    fn to_specialization_enum_box(self: Box<Self>) -> SpecializationEnum {
        self.to_specialization_enum()
    }
}
impl Specialize for AnyPolicyBB {
    fn specialize_leaf(b: Box<dyn AnyPolicyTrait>) -> Result<Self, Box<dyn AnyPolicyTrait>> {
        //let x = b.transfer()?;
        if TypeId::of::<Box<dyn AnyPolicyTrait>>() == TypeId::of::<Box<dyn AnyPolicyTrait>>() {
            let b: Box<dyn Any> = Box::new(b);
            let b = b.downcast().unwrap();
            Ok(AnyPolicyDyn { policy:  *b })
        } else {
            Err(b)
        }
    }
    fn specialize_and(b1: Box<SpecializationEnum>, b2: Box<SpecializationEnum>) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<Self>();
        let r2 = b2.specialize::<Self>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(<dyn AnyPolicyTrait>::and_policy(PolicyAnd::new(p1, p2))),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) =>
                Err((Box::new(p1.to_specialization_enum().normalize()), Box::new(e2))),
            (Err(e1), Ok(p2)) =>
                Err((Box::new(e1), Box::new(p2.to_specialization_enum().normalize()))),
        }
    }
    fn specialize_or(b1: Box<SpecializationEnum>, b2: Box<SpecializationEnum>) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<Self>();
        let r2 = b2.specialize::<Self>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(<dyn AnyPolicyTrait>::or_policy(PolicyOr::new(p1, p2))),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) =>
                Err((Box::new(p1.to_specialization_enum().normalize()), Box::new(e2))),
            (Err(e1), Ok(p2)) =>
                Err((Box::new(e1), Box::new(p2.to_specialization_enum().normalize()))),
        }
    }
    fn specialize_option(b: Option<Box<SpecializationEnum>>) -> Result<Self, Option<Box<SpecializationEnum>>> {
        match b {
            None => Ok(AnyPolicyDyn::default()),
            Some(b) => {
                match b.specialize::<Self>() {
                    Ok(p) => Ok(p),
                    Err(b) => Err(Some(Box::new(b))),
                }
            },
        }
    }
}

// AnyPolicyDyn is Clone if it obligates trait object to be Clone as well.
impl<P: PolicyDyn + DynClone + ?Sized> Clone for AnyPolicyDyn<P> {
    fn clone(&self) -> Self {
        Self { policy: dyn_clone::clone_box(self.policy.as_ref()) }
    }
}

// Convert to parent.
impl<P: PolicyDyn + ?Sized> AnyPolicyDyn<P> {
    pub fn upcast_super_box(self) -> AnyPolicyDyn<dyn AnyPolicyTrait> {
        let policy = self.policy;
        AnyPolicyDyn { policy: policy.upcast_super_boxed() }
    }
}

// Can always cast no_policy to AnyPolicyDyn of any kind.
impl<P: PolicyDyn + ?Sized> Default for AnyPolicyDyn<P> {
    fn default() -> Self {
        Self { policy: P::no_policy() }
    }
}

#[cfg(test)]
mod tests {
    use crate::policy::{AnyPolicyBB, AnyPolicyCC, AnyPolicySS, NoPolicy};

    #[test]
    fn any_policy_in_any_policy() {
        let any_policy = AnyPolicyCC::new(NoPolicy {} );
        assert!(any_policy.is::<NoPolicy>());
        assert!(any_policy.clone().specialize_top::<NoPolicy>().is_ok());

        let any_policy2 = AnyPolicyCC::new(any_policy.clone());
        assert!(any_policy2.is::<NoPolicy>());
        assert!(any_policy2.specialize_top::<NoPolicy>().is_ok());

        let any_policy3 = AnyPolicyBB::new(any_policy);
        assert!(any_policy3.is::<NoPolicy>());
        assert!(any_policy3.specialize_top::<NoPolicy>().is_ok());


        let any_policy = AnyPolicySS::new(NoPolicy {} );
        assert!(any_policy.is::<NoPolicy>());

        let any_policy2 = AnyPolicySS::new(any_policy);
        assert!(any_policy2.is::<NoPolicy>());
        assert!(any_policy2.specialize_top::<NoPolicy>().is_ok());

        let any_policy = AnyPolicySS::new(NoPolicy {} );
        assert!(any_policy.is::<NoPolicy>());

        let any_policy3 = AnyPolicyBB::new(any_policy);
        assert!(any_policy3.is::<NoPolicy>());
        assert!(any_policy3.specialize_top::<NoPolicy>().is_ok());
    }
}