use crate::policy::{
    AnyPolicy, AnyPolicyDyn, NoPolicy, OptionPolicy, Policy, PolicyAnd, PolicyDyn, PolicyOr,
    RefPolicy, SpecializationEnum, Specialize,
};
use crate::policy::{NotAPolicyContainer, ReflectiveOwned};
use crate::testing::TestPolicy;
use std::any::Any;

// Leafs.
impl<P: Policy + Any + NotAPolicyContainer> Specialize for P {
    fn specialize_leaf(b: Box<dyn AnyPolicyDyn>) -> Result<Self, Box<dyn AnyPolicyDyn>>
    where
        Self: Sized,
    {
        if b.upcast_any().is::<P>() {
            Ok(*b.upcast_any_box().downcast::<P>().unwrap())
        } else {
            Err(b)
        }
    }
}

// RefPolicy with a 'static lifetime is essentially a leaf!
impl<P: Policy + Any> Specialize for RefPolicy<'static, P> {
    fn specialize_leaf(b: Box<dyn AnyPolicyDyn>) -> Result<Self, Box<dyn AnyPolicyDyn>> {
        if b.upcast_any().is::<Self>() {
            Ok(*b.upcast_any_box().downcast().unwrap())
        } else {
            Err(b)
        }
    }
}

// Ands and Ors.
impl<P1: Policy + Specialize, P2: Policy + Specialize> Specialize for PolicyAnd<P1, P2> {
    fn specialize_and(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<P1>();
        let r2 = b2.specialize::<P2>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(PolicyAnd::new(p1, p2)),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) => Err((
                Box::new(Box::new(p1).reflect_static().normalize()),
                Box::new(e2),
            )),
            (Err(e1), Ok(p2)) => Err((
                Box::new(e1),
                Box::new(Box::new(p2).reflect_static().normalize()),
            )),
        }
    }
}
impl<P1: Specialize, P2: Specialize> Specialize for PolicyOr<P1, P2> {
    fn specialize_or(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<P1>();
        let r2 = b2.specialize::<P2>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(PolicyOr::new(p1, p2)),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) => Err((
                Box::new(Box::new(p1).reflect_static().normalize()),
                Box::new(e2),
            )),
            (Err(e1), Ok(p2)) => Err((
                Box::new(e1),
                Box::new(Box::new(p2).reflect_static().normalize()),
            )),
        }
    }
}

// Option Policy.
impl<P: Policy + Specialize> Specialize for OptionPolicy<P> {
    fn specialize_leaf(b: Box<dyn AnyPolicyDyn>) -> Result<Self, Box<dyn AnyPolicyDyn>> {
        let any = b.upcast_any();
        if any.is::<NoPolicy>() {
            Ok(OptionPolicy::NoPolicy)
        } else if any.is::<P>() {
            P::specialize_leaf(b).map(OptionPolicy::Policy)
        } else {
            Err(b)
        }
    }
    fn specialize_option(
        b: Option<Box<SpecializationEnum>>,
    ) -> Result<Self, Option<Box<SpecializationEnum>>> {
        match b {
            None => Ok(OptionPolicy::NoPolicy),
            Some(b) => match b.specialize::<P>() {
                Ok(p) => Ok(OptionPolicy::Policy(p)),
                Err(b) => Err(Some(Box::new(b))),
            },
        }
    }
}

// AnyPolicyDyn.
impl Specialize for AnyPolicy {
    fn specialize_leaf(b: Box<dyn AnyPolicyDyn>) -> Result<Self, Box<dyn AnyPolicyDyn>> {
        Ok(AnyPolicy::from_inner(b))
    }
    fn specialize_and(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<Self>();
        let r2 = b2.specialize::<Self>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(PolicyDyn::and_policy(PolicyAnd::new(p1, p2))),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) => Err((Box::new(p1.reflect_owned().normalize()), Box::new(e2))),
            (Err(e1), Ok(p2)) => Err((Box::new(e1), Box::new(p2.reflect_owned().normalize()))),
        }
    }
    fn specialize_or(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        let r1 = b1.specialize::<Self>();
        let r2 = b2.specialize::<Self>();
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => Ok(PolicyDyn::or_policy(PolicyOr::new(p1, p2))),
            (Err(e1), Err(e2)) => Err((Box::new(e1), Box::new(e2))),
            (Ok(p1), Err(e2)) => Err((Box::new(p1.reflect_owned().normalize()), Box::new(e2))),
            (Err(e1), Ok(p2)) => Err((Box::new(e1), Box::new(p2.reflect_owned().normalize()))),
        }
    }
    fn specialize_option(
        b: Option<Box<SpecializationEnum>>,
    ) -> Result<Self, Option<Box<SpecializationEnum>>> {
        match b {
            None => Ok(AnyPolicy::default()),
            Some(b) => match b.specialize::<Self>() {
                Ok(p) => Ok(p),
                Err(b) => Err(Some(Box::new(b))),
            },
        }
    }
}

// TestPolicy.
impl<P: Policy + Specialize> Specialize for TestPolicy<P> {
    fn specialize_leaf(b: Box<dyn AnyPolicyDyn>) -> Result<Self, Box<dyn AnyPolicyDyn>> {
        Ok(TestPolicy::new(P::specialize_leaf(b)?))
    }
    fn specialize_and(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        Ok(TestPolicy::new(P::specialize_and(b1, b2)?))
    }
    fn specialize_or(
        b1: Box<SpecializationEnum>,
        b2: Box<SpecializationEnum>,
    ) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> {
        Ok(TestPolicy::new(P::specialize_or(b1, b2)?))
    }
    fn specialize_option(
        b: Option<Box<SpecializationEnum>>,
    ) -> Result<Self, Option<Box<SpecializationEnum>>> {
        Ok(TestPolicy::new(P::specialize_option(b)?))
    }
}
