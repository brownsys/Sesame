use crate::policy::{AnyPolicyDyn, PolicyDyn};
use crate::sesame_type::dyns::SesameDyn;
use crate::sesame_type::r#enum::SesameTypeEnum;
use std::any::Any;

pub trait SesameTypeOut {
    // Unboxed form of struct
    type Out;
}

// Public: client code should derive this for structs that they want to unbox, fold, or pass to
// sandboxes.
pub trait SesameType<T: SesameDyn + ?Sized = dyn Any, P: PolicyDyn + ?Sized = dyn AnyPolicyDyn>:
    SesameTypeOut
{
    fn to_enum(self) -> SesameTypeEnum<T, P>;
    fn from_enum(e: SesameTypeEnum<T, P>) -> Result<Self, ()>
    where
        Self: Sized;
    fn out_from_enum(e: SesameTypeEnum<T, P>) -> Result<Self::Out, ()>;
}
