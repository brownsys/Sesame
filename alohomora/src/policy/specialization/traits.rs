use std::any::Any;
use std::boxed::Box;
use crate::fold_in::NotAPolicyContainer;
use crate::policy::{Policy, AnyPolicyTrait, SpecializationEnum};

pub trait Specializable: Policy + Any {
    // TODO(babman): we do not need to have these here.
    //               we should have a generic reflect() function
    //               on Policy, which can configured to be
    //               Owning or Ref-ing.
    //               With max lifetime that Self outlives.
    // Allows us to do reflection on policy types.
    fn to_specialization_enum(self) -> SpecializationEnum;
    fn to_specialization_enum_box(self: Box<Self>) -> SpecializationEnum;
    // API that application developers use to specialize.
    fn specialize<P: Specialize>(self) -> Result<P, SpecializationEnum> where Self: Sized {
        let e = self.reflect_owned().normalize();
        e.specialize::<P>()
    }
}

type BoxEnum = Box<SpecializationEnum>;

pub trait Specialize : Specializable {
    // Constructs instances of the target type.
    #[inline]
    fn specialize_leaf(b: Box<dyn AnyPolicyTrait>) -> Result<Self, Box<dyn AnyPolicyTrait>> where Self: Sized {
        Err(b)
    }
    #[inline]
    fn specialize_and(b1: BoxEnum, b2: BoxEnum) -> Result<Self, (BoxEnum, BoxEnum)> where Self: Sized {
        Err((b1, b2))
    }
    #[inline]
    fn specialize_or(b1: BoxEnum, b2: BoxEnum) -> Result<Self, (BoxEnum, BoxEnum)> where Self: Sized {
        Err((b1, b2))
    }
    #[inline]
    fn specialize_option(b: Option<BoxEnum>) -> Result<Self, Option<BoxEnum>> where Self: Sized {
        Err(b)
    }
}

impl<P: Policy + Any + NotAPolicyContainer> Specializable for P {
    fn to_specialization_enum(self) ->SpecializationEnum
    where
        Self: Sized
    {
        SpecializationEnum::Leaf(Box::new(self))
    }
    fn to_specialization_enum_box(self: Box<Self>) -> SpecializationEnum
    where
        Self: Sized
    {
        self.to_specialization_enum()
    }
}

impl<P: Policy + Any + NotAPolicyContainer> Specialize for P {
    fn specialize_leaf(b: Box<dyn AnyPolicyTrait>) -> Result<Self, Box<dyn AnyPolicyTrait>> where Self: Sized {
        if b.upcast_any().is::<P>() {
            Ok(*b.upcast_any_box().downcast::<P>().unwrap())
        } else {
            Err(b)
        }
    }
}