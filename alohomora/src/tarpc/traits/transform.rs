use crate::bbox::BBox;
use crate::policy::{Policy, PolicyFrom, PolicyInto};
use crate::tarpc::context::TahiniContext;

use super::TahiniType;

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
//

impl<P> TahiniTransformFrom<P> for P {
    fn transform_from(other: P, _context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        Ok(other)
    }
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
