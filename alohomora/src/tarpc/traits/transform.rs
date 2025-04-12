use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::bbox::BBox;
use crate::policy::{Policy, PolicyFrom, PolicyInto};
use crate::tarpc::context::TahiniContext;
use crate::tarpc::{TahiniEnum, TahiniVariantsEnum};

use super::{TahiniError, TahiniType};

///Contains either an Uninitialized context from the wire, or an initialized one for local
///transformation
#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum EitherTahiniContext {
    Uninitialized,
    Initialized(TahiniContext),
}

impl<'a> TahiniType for EitherTahiniContext {
    fn to_tahini_enum(&self) -> TahiniEnum {
        match self {
            Self::Uninitialized => TahiniEnum::Enum(
                "EitherTahiniContext",
                0,
                "Uninitialized",
                TahiniVariantsEnum::Unit,
            ),
            Self::Initialized(context) => TahiniEnum::Enum(
                "EitherTahiniContext",
                1,
                "Initialized",
                TahiniVariantsEnum::NewType(Box::new(context.to_tahini_enum())),
            ),
        }
    }
}

///Contains some transformable data and some context for the transformation.
#[derive(Deserialize, Clone)]
pub struct Fromable<T: TahiniType> {
    pub(crate) context: EitherTahiniContext,
    data: T,
}

impl<T: TahiniType> Fromable<T> {
    pub fn new(data: T) -> Self {
        Self {
            context: EitherTahiniContext::Uninitialized,
            data,
        }
    }

    ///Adds some transformation-specific context. This separation is required so that the
    ///transformation can be called from application code, but the context can be setup in a secure
    ///way
    pub(crate) fn add_context(&mut self, context: TahiniContext) {
        self.context = EitherTahiniContext::Initialized(context)
    }

    ///Transforms from TahiniType into some local type if transformation allows it
    pub fn transform_into<U: TahiniTransformFrom<T>>(self) -> Result<U, String> {
        match self.context {
            EitherTahiniContext::Uninitialized => Err("Context was not initialized".to_string()),
            EitherTahiniContext::Initialized(some_ctxt) => U::transform_from(self.data, &some_ctxt),
        }
    }
}

impl<
        T: TahiniType + Clone + 'static,
        E: std::error::Error + Clone + Send + TahiniError + 'static,
    > Fromable<Result<T, E>>
{
    pub fn transpose(self) -> Result<Fromable<T>, E> {
        match self.data {
            Ok(d) => Ok(Fromable {
                context: self.context,
                data: d,
            }),
            Err(e) => Err(e),
        }
    }
}

impl<T: TahiniType> TahiniType for Fromable<T> {
    fn to_tahini_enum(&self) -> TahiniEnum {
        let mut map = HashMap::new();
        map.insert("context", self.context.to_tahini_enum());
        map.insert("data", self.data.to_tahini_enum());
        TahiniEnum::Struct("Fromable", map)
    }
}

///Developers should implement this trait when parsing an object of a remote type into one that is
///handled locally.
///Note that it is assumed converting to a local type is always safe. As such, no distinction on
///the data flow is made on the Context here.
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

// impl<T> TahiniTransformFrom<T> for T {
//     #[inline]
//     fn transform_from(other: T, _context: &TahiniContext) -> Result<Self, String>
//     where
//         Self: Sized,
//     {
//         Ok(other)
//     }
// }

impl<T, SourcePolicy: PolicyInto<TargetPolicy>, TargetPolicy: Policy>
    TahiniTransformInto<BBox<T, TargetPolicy>> for BBox<T, SourcePolicy>
{
    fn transform_into(self, context: &TahiniContext) -> Result<BBox<T, TargetPolicy>, String> {
        let (t, p) = self.consume();
        Ok(BBox::new(t, p.into_policy(context)?))
    }
}

impl<T, TargetPolicy: PolicyFrom<SourcePolicy>, SourcePolicy: Policy>
    TahiniTransformFrom<BBox<T, SourcePolicy>> for BBox<T, TargetPolicy>
{
    fn transform_from(other: BBox<T, SourcePolicy>, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        let (t, p) = other.consume();
        Ok(BBox::new(t, TargetPolicy::from_policy(p, context)?))
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

impl<SourceType, TargetType: TahiniTransformFrom<SourceType>, E: std::error::Error>
    TahiniTransformFrom<Result<SourceType, E>> for Result<TargetType, E>
{
    fn transform_from(other: Result<SourceType, E>, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        match other {
            Ok(src) => Ok(Ok(TargetType::transform_from(src, context)?)),
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

impl<SourceType, TargetType: TahiniTransformFrom<SourceType>> TahiniTransformFrom<Vec<SourceType>>
    for Vec<TargetType>
{
    fn transform_from(other: Vec<SourceType>, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        other
            .into_iter()
            .map(|x| TargetType::transform_from(x, context))
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

impl<SourceType, TargetType: TahiniTransformFrom<SourceType>>
    TahiniTransformFrom<Option<SourceType>> for Option<TargetType>
{
    fn transform_from(other: Option<SourceType>, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        other
            .map(|some| TargetType::transform_from(some, context))
            .transpose()
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

        impl TahiniTransformFrom<$ty> for $ty {
            #[inline]
            fn transform_from(other: $ty, _context: &TahiniContext) -> Result<Self, String> {
                Ok(other)
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
