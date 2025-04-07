use crate::bbox::BBox;
use crate::policy::{Policy, PolicyFrom, PolicyInto};
use crate::tarpc::context::TahiniContext;

use super::TahiniType;

///Developers should implement this trait when parsing an object of a remote type into one that is
///handled locally.
///Note that it is assumed converting to a local type is always safe. As such, no distinction on
///the data flow is made on the Context here.
// #[rustc_specialization_trait]
pub trait TahiniTransformFrom<SourceType> {
    fn transform_from(other: SourceType, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized;
}

///Developers should implement this type when converting a local type to one provided by a remote
///RPC library. Implementing this trait only makes sense if you expect the type to be "consumed" by
///the RPC. The TahiniContext is assumed to always be on the egress here.
pub trait TahiniTransformInto<TargetType> {
    fn transform_into(self, context: &TahiniContext) -> Result<TargetType, String>;
}

// impl<P, TargetType: TahiniTransformFrom<P>> TahiniTransformInto<TargetType> for P {
//     fn transform_into(self, context: &TahiniContext) -> Result<TargetType, String> {
//         TargetType::transform_from(self, context)
//     }
// }

impl<TargetType, SourceType: TahiniTransformInto<TargetType>> TahiniTransformFrom<SourceType>
    for TargetType
{
    fn transform_from(other: SourceType, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        other.transform_into(context)
    }
}

//
macro_rules! register_self_transform {
    ($ty: ty) => {
        impl TahiniTransformInto<$ty> for $ty {
            fn transform_into(self, context: &TahiniContext) -> Result<$ty, String> {
                Ok(self)
            }
        }
    };
}

impl<T, SourcePolicy: PolicyInto<TargetPolicy>, TargetPolicy: Policy>
    TahiniTransformInto<BBox<T, TargetPolicy>> for BBox<T, SourcePolicy>
{
    fn transform_into(self, context: &TahiniContext) -> Result<BBox<T, TargetPolicy>, String> {
        let (t, p) = self.consume();
        Ok(BBox::new(t, p.into_policy(context)?))
    }
}

impl<TargetType, SourceType: TahiniTransformInto<TargetType>, E: std::error::Error>
    TahiniTransformInto<Result<TargetType, E>> for Result<SourceType, E>
{
    fn transform_into(self, context: &TahiniContext) -> Result<Result<TargetType, E>, String> {
        match self {
            Ok(src) => Ok(Ok(src.transform_into(context)?)),
            Err(e) => Ok(Err(e)),
        }
    }
}

impl<TargetType, SourceType: TahiniTransformInto<TargetType>> TahiniTransformInto<Vec<TargetType>>
    for Vec<SourceType>
{
    fn transform_into(self, context: &TahiniContext) -> Result<Vec<TargetType>, String> {
        self.into_iter()
            .map(|x| x.transform_into(context))
            .collect()
    }
}

impl<TargetType, SourceType: TahiniTransformInto<TargetType>>
    TahiniTransformInto<Option<TargetType>> for Option<SourceType>
{
    fn transform_into(self, context: &TahiniContext) -> Result<Option<TargetType>, String> {
        self.map(|some| some.transform_into(context)).transpose()
    }
}

macro_rules! transform_type_tuple_impl {
  ($([$SourceType:tt,$DestType:tt,$i:tt]),*) => (
    impl<$($DestType: TahiniType),*, $($SourceType: TahiniTransformInto<$DestType>,)*> TahiniTransformInto<($($DestType,)*)> for ($($SourceType,)*) {
        fn transform_into(self, context: &TahiniContext) -> Result<($($DestType,)*), String> {
            Ok(($(self.$i.transform_into(context)?),*,))
        }
    }
  );
}

transform_type_tuple_impl!([Source1, DestType1, 0]);
transform_type_tuple_impl!([SourceType0, DestType0, 0], [SourceType1, DestType1, 1]);
transform_type_tuple_impl!(
    [SourceType0, DestType0, 0],
    [SourceType1, DestType1, 1],
    [SourceType2, DestType2, 2]
);
transform_type_tuple_impl!(
    [SourceType0, DestType0, 0],
    [SourceType1, DestType1, 1],
    [SourceType2, DestType2, 2],
    [SourceType3, DestType3, 3]
);
transform_type_tuple_impl!(
    [SourceType0, DestType0, 0],
    [SourceType1, DestType1, 1],
    [SourceType2, DestType2, 2],
    [SourceType3, DestType3, 3],
    [SourceType4, DestType4, 4]
);

macro_rules! transform_type_prim_impl {
    ($ty: ty) => {
        impl TahiniTransformInto<$ty> for $ty {
            fn transform_into(self, _context: &TahiniContext) -> Result<$ty, String> {
                Ok(self)
            }
        }
    };
}

transform_type_prim_impl!(u8);
transform_type_prim_impl!(u16);
transform_type_prim_impl!(u32);
transform_type_prim_impl!(i8);
transform_type_prim_impl!(i16);
transform_type_prim_impl!(i32);
transform_type_prim_impl!(usize);
transform_type_prim_impl!(String);
transform_type_prim_impl!(bool);
transform_type_prim_impl!(&'static str);
