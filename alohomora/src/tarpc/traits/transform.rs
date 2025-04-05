use crate::bbox::BBox;
use crate::policy::{Policy, PolicyFrom, PolicyInto};
use crate::tarpc::context::TahiniContext;

use super::TahiniType;

pub trait TahiniTransformFrom<SourceType> {
    fn transform_from(other: SourceType, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized;
}

pub trait TahiniTransformInto<TargetType> {
    fn transform_into(self, context: &TahiniContext) -> Result<TargetType, String>;
}

impl<P, TargetType: TahiniTransformFrom<P>> TahiniTransformInto<TargetType> for P {
    fn transform_into(self, context: &TahiniContext) -> Result<TargetType, String> {
        TargetType::transform_from(self, context)
    }
}

impl<T, SourcePolicy: PolicyInto<TargetPolicy>, TargetPolicy: Policy>
    TahiniTransformFrom<BBox<T, SourcePolicy>> for BBox<T, TargetPolicy>
{
    fn transform_from(
        other: BBox<T, SourcePolicy>,
        context: &TahiniContext,
    ) -> Result<Self, String> {
        let (t, p) = other.consume();
        Ok(BBox::new(t, p.into_policy(context)?))
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
            Ok(ok) => Ok(Ok(ok.transform_into(context)?)),
            Err(e) => Ok(Err(e)),
        }
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
            .map(|x| x.transform_into(context))
            .collect()
    }
}

impl<SourceType, TargetType: TahiniTransformFrom<SourceType>>
    TahiniTransformFrom<Option<SourceType>> for Option<TargetType>
{
    fn transform_from(other: Option<SourceType>, context: &TahiniContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        other.map(|some| some.transform_into(context)).transpose()
    }
}
