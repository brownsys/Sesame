use std::any::Any;
use crate::policy::{AnyPolicyTrait, PolicyDyn};
use crate::sesame_type::r#enum::SesameTypeEnum;
use crate::sesame_type::dyns::SesameDyn;

// Public: client code should derive this for structs that they want to unbox, fold, or pass to
// sandboxes.
pub trait SesameType<T: SesameDyn + ?Sized = dyn Any, P: PolicyDyn + ?Sized = dyn AnyPolicyTrait> {
    type Out; // Unboxed form of struct
    fn to_enum(self) -> SesameTypeEnum<T, P>;
    fn from_enum(e: SesameTypeEnum<T, P>) -> Result<Self::Out, ()>;
}