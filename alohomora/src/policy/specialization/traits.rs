use std::any::Any;
use std::boxed::Box;

use crate::policy::{Policy, AnyPolicyTrait, SpecializationEnum};

// Allows upgrading Box<dyn Policy + 'static> to Box<dyn Policy + Any>.
mod private {
    use crate::policy::Policy;

    pub trait Sealed {}
    impl<P: Policy + Sized> Sealed for P {}
}
pub trait UpgradableToAny : private::Sealed {
    fn upgrade_to_any(&self) -> &dyn AnyPolicyTrait where Self: 'static;
    fn upgrade_to_any_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait> where Self: 'static;
}
impl<P: Policy + Sized> UpgradableToAny for P {
    fn upgrade_to_any(&self) -> &dyn AnyPolicyTrait where Self: 'static { self }
    fn upgrade_to_any_box(self: Box<Self>) -> Box<dyn AnyPolicyTrait> where Self: 'static { self }
}

// Things we can call .Specialize() on (i.e. source of specialization).
pub trait Specializable: Policy + Any {
    fn specialize<P: Specialize>(self) -> Result<P, SpecializationEnum> where Self: Sized;
}
impl<P: Policy + Any> Specializable for P {
    // API that application developers use to specialize.
    fn specialize<P2: Specialize>(self) -> Result<P2, SpecializationEnum> where Self: Sized {
        let e: SpecializationEnum = Box::new(self).reflect_static().normalize();
        e.specialize::<P2>()
    }
}

// Types we can specialize to (i.e. destination of specialization).
pub trait Specialize : Specializable {
    // Constructs instances of the target type.
    #[inline]
    fn specialize_leaf(b: Box<dyn AnyPolicyTrait>) -> Result<Self, Box<dyn AnyPolicyTrait>> where Self: Sized {
        Err(b)
    }
    #[inline]
    fn specialize_and(b1: Box<SpecializationEnum>, b2: Box<SpecializationEnum>) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> where Self: Sized {
        Err((b1, b2))
    }
    #[inline]
    fn specialize_or(b1: Box<SpecializationEnum>, b2: Box<SpecializationEnum>) -> Result<Self, (Box<SpecializationEnum>, Box<SpecializationEnum>)> where Self: Sized {
        Err((b1, b2))
    }
    #[inline]
    fn specialize_option(b: Option<Box<SpecializationEnum>>) -> Result<Self, Option<Box<SpecializationEnum>>> where Self: Sized {
        Err(b)
    }
}