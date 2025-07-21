use std::any::Any;

use erased_serde::Serialize;

use crate::sesame_type::r#enum::{SesameTypeEnum, SesameTypeEnumDyn};

// Traits that we care about preserving inside SesameType.
pub trait AnySerialize: Serialize + Any {
    fn upcast_any(&self) -> &dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
}
impl<T: Serialize + Any> AnySerialize for T {
    fn upcast_any(&self) -> &dyn Any {
        self
    }
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any> {
        Box::new(*self)
    }
}

// Public: client code should derive this for structs that they want to unbox, fold, or pass to
// sandboxes.
pub trait SesameTypeDyn<T: ?Sized> {
    type Out; // Unboxed form of struct
    fn to_enum(self) -> SesameTypeEnumDyn<T>;
    fn from_enum(e: SesameTypeEnumDyn<T>) -> Result<Self::Out, ()>;
}

// Sealed alias for the common case where we just need to be able to go from T to T::Out.
mod private {
    pub trait Sealed {}
    impl<T: super::SesameTypeDyn<dyn super::Any>> Sealed for T {}
}

pub trait SesameType : private::Sealed {
    type Out; // Unboxed form of struct
    fn to_enum(self) -> SesameTypeEnum;
    fn from_enum(e: SesameTypeEnum) -> Result<Self::Out, ()>;
}
impl<T: SesameTypeDyn<dyn Any>> SesameType for T {
    type Out = T::Out;
    fn to_enum(self) -> SesameTypeEnum {
        <T as SesameTypeDyn<dyn Any>>::to_enum(self)
    }
    fn from_enum(e: SesameTypeEnum) -> Result<Self::Out, ()> {
        <T as SesameTypeDyn<dyn Any>>::from_enum(e)
    }
}