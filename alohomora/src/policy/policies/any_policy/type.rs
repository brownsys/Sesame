use crate::context::UnprotectedContext;
use crate::policy::{
    AnyPolicyCloneDyn, AnyPolicyDyn, AnyPolicySerializeDyn, Policy, PolicyDynInto, Reason,
};
use crate::policy::{AnyPolicyable, PolicyDyn, PolicyDynRelation};
use dyn_clone::DynClone;
use serde::Serialize;
use std::any::TypeId;

// Aliases for ease of use.
pub type AnyPolicyClone = AnyPolicy<dyn AnyPolicyCloneDyn>;
pub type AnyPolicySerialize = AnyPolicy<dyn AnyPolicySerializeDyn>;

// Type-erased AnyPolicy that can pose dyn trait object obligations
// on the policy inside of it.
#[derive(Serialize)]
pub struct AnyPolicy<P: PolicyDyn + ?Sized = dyn AnyPolicyDyn> {
    policy: Box<P>,
}
impl<P: PolicyDyn + ?Sized> AnyPolicy<P> {
    // Constructor.
    pub fn new<T: AnyPolicyable>(p: T) -> Self
    where
        P: PolicyDynRelation<T>,
    {
        Self {
            policy: P::boxed_dyn(p),
        }
    }

    // Check what the concrete type is.
    pub fn is<T: AnyPolicyable>(&self) -> bool
    where
        P: PolicyDynRelation<T>,
    {
        TypeId::of::<T>() == self.policy.upcast_ref().type_id()
    }

    // Specialize the type down to the concrete type (down cast).
    pub fn specialize_top<T: AnyPolicyable>(self) -> Result<T, String>
    where
        P: PolicyDynRelation<T>,
    {
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
    where
        P: PolicyDynRelation<T>,
    {
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
    pub fn mut_inner(&mut self) -> &mut P {
        self.policy.as_mut()
    }
    pub fn into_inner(self) -> Box<P> {
        self.policy
    }
    pub fn from_inner(policy: Box<P>) -> Self {
        Self { policy }
    }

    // Convert between dyns.
    pub fn convert_to<PDyn: PolicyDyn + ?Sized>(self) -> AnyPolicy<PDyn>
    where
        P: PolicyDynInto<PDyn>,
    {
        AnyPolicy {
            policy: self.policy.policy_dyn_into_boxed(),
        }
    }
}

// AnyPolicyDyn is a Policy.
impl<P: PolicyDyn + ?Sized> Policy for AnyPolicy<P> {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.upcast_pref().name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.policy.upcast_pref().check(context, reason)
    }
}

// AnyPolicyDyn is Clone if it obligates trait object to be Clone as well.
impl<P: PolicyDyn + DynClone + ?Sized> Clone for AnyPolicy<P> {
    fn clone(&self) -> Self {
        Self {
            policy: dyn_clone::clone_box(self.policy.as_ref()),
        }
    }
}

// Convert to parent.
impl<P: PolicyDyn + ?Sized> AnyPolicy<P> {
    pub fn upcast_super_box(self) -> AnyPolicy<dyn AnyPolicyDyn> {
        let policy = self.policy;
        AnyPolicy {
            policy: policy.upcast_super_boxed(),
        }
    }
}

// Can always cast no_policy to AnyPolicyDyn of any kind.
impl<P: PolicyDyn + ?Sized> Default for AnyPolicy<P> {
    fn default() -> Self {
        Self {
            policy: P::no_policy(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::policy::{AnyPolicy, AnyPolicyClone, AnyPolicySerialize, NoPolicy};

    #[test]
    fn any_policy_in_any_policy() {
        let any_policy = AnyPolicyClone::new(NoPolicy {});
        assert!(any_policy.is::<NoPolicy>());
        assert!(any_policy.clone().specialize_top::<NoPolicy>().is_ok());

        let any_policy2 = AnyPolicyClone::new(any_policy.clone());
        assert!(any_policy2.is::<NoPolicy>());
        assert!(any_policy2.specialize_top::<NoPolicy>().is_ok());

        let any_policy3: AnyPolicy = AnyPolicy::new(any_policy);
        assert!(any_policy3.is::<NoPolicy>());
        assert!(any_policy3.specialize_top::<NoPolicy>().is_ok());

        let any_policy = AnyPolicySerialize::new(NoPolicy {});
        assert!(any_policy.is::<NoPolicy>());

        let any_policy2 = AnyPolicySerialize::new(any_policy);
        assert!(any_policy2.is::<NoPolicy>());
        assert!(any_policy2.specialize_top::<NoPolicy>().is_ok());

        let any_policy = AnyPolicySerialize::new(NoPolicy {});
        assert!(any_policy.is::<NoPolicy>());

        let any_policy3: AnyPolicy = AnyPolicy::new(any_policy);
        assert!(any_policy3.is::<NoPolicy>());
        assert!(any_policy3.specialize_top::<NoPolicy>().is_ok());
    }
}
