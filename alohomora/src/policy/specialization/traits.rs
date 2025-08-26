use std::any::Any;
use std::boxed::Box;

use crate::policy::{AnyPolicyDyn, Policy, SpecializationEnum};

// Allows upgrading Box<dyn Policy + 'static> to Box<dyn Policy + Any>.
mod private {
    use crate::policy::Policy;

    pub trait Sealed {}
    impl<P: Policy + Sized> Sealed for P {}
}
pub trait UpgradableToAny: private::Sealed {
    fn upgrade_to_any(&self) -> &dyn AnyPolicyDyn
    where
        Self: 'static;
    fn upgrade_to_any_box(self: Box<Self>) -> Box<dyn AnyPolicyDyn>
    where
        Self: 'static;
}
impl<P: Policy + Sized> UpgradableToAny for P {
    fn upgrade_to_any(&self) -> &dyn AnyPolicyDyn
    where
        Self: 'static,
    {
        self
    }
    fn upgrade_to_any_box(self: Box<Self>) -> Box<dyn AnyPolicyDyn>
    where
        Self: 'static,
    {
        self
    }
}

// Things we can call .Specialize() on (i.e. source of specialization).
pub trait Specializable: Policy + Any {
    fn specialize<P: Specialize>(self) -> Result<P, SpecializationEnum>
    where
        Self: Sized;
}
impl<P: Policy + Any> Specializable for P {
    // API that application developers use to specialize.
    fn specialize<P2: Specialize>(self) -> Result<P2, SpecializationEnum>
    where
        Self: Sized,
    {
        let e: SpecializationEnum = Box::new(self).reflect_static().normalize();
        e.specialize::<P2>()
    }
}

// Types we can specialize to (i.e. destination of specialization).
pub trait Specialize: Specializable {
    // Constructs instances of the target type.
    #[inline]
    fn specialize_leaf(b: Box<dyn AnyPolicyDyn>) -> Result<Self, Box<dyn AnyPolicyDyn>>
    where
        Self: Sized,
    {
        Err(b)
    }
    #[inline]
    fn specialize_and(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)>
    where
        Self: Sized,
    {
        Err((b1, b2))
    }
    #[inline]
    fn specialize_or(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)>
    where
        Self: Sized,
    {
        Err((b1, b2))
    }
    #[inline]
    fn specialize_option(
        b: Option<Box<SpecializationEnum>>,
    ) -> Result<Self, Option<Box<SpecializationEnum>>>
    where
        Self: Sized,
    {
        Err(b)
    }
}

// TODO(babman): add missing tests.
#[cfg(test)]
mod tests {
    use crate::context::UnprotectedContext;
    use crate::policy::{
        AnyPolicy, AnyPolicyCloneDyn, Join, NoPolicy, Policy, PolicyAnd, Reason, RefPolicy,
        ReflectiveOwned, Specializable,
    };

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct UnjoinablePolicy {
        pub v: u32,
    }
    impl Join for UnjoinablePolicy {}
    impl Policy for UnjoinablePolicy {
        fn name(&self) -> String {
            format!("Unjoinable(v: {})", self.v)
        }
        fn check(&self, _context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            true
        }
        // This policy is unjoinable.
    }

    #[test]
    fn my_special_test() {
        let policy = AnyPolicy::<dyn AnyPolicyCloneDyn>::new(
            AnyPolicy::<dyn AnyPolicyCloneDyn>::new(PolicyAnd::new(
                AnyPolicy::<dyn AnyPolicyCloneDyn>::new(PolicyAnd::new(
                    AnyPolicy::<dyn AnyPolicyCloneDyn>::new(UnjoinablePolicy { v: 0 }),
                    AnyPolicy::<dyn AnyPolicyCloneDyn>::new(UnjoinablePolicy { v: 50 }),
                )),
                UnjoinablePolicy { v: 20 },
            )),
        );

        println!("{}", policy.name());

        type Stacked = PolicyAnd<PolicyAnd<AnyPolicy, UnjoinablePolicy>, UnjoinablePolicy>;
        let e = policy.reflect_owned().normalize();
        let p = e.specialize::<Stacked>().map_err(|_| ()).unwrap();
        println!("{}", p.name());
    }

    #[test]
    fn specialize_policy_ref() {
        let policy = NoPolicy {};
        let policy2 = UnjoinablePolicy { v: 50 };
        let policy3 = PolicyAnd::new(policy, policy2);
        let refpolicy: RefPolicy<'static, PolicyAnd<NoPolicy, UnjoinablePolicy>> =
            RefPolicy::new(unsafe { std::mem::transmute(&policy3) });

        let anypolicy: AnyPolicy = AnyPolicy::new(refpolicy);
        type Reffed = RefPolicy<'static, PolicyAnd<NoPolicy, UnjoinablePolicy>>;
        let reffed = anypolicy.specialize::<Reffed>().map_err(|_| ()).unwrap();
        assert_eq!(reffed.policy().policy2().v, 50);
    }
}
