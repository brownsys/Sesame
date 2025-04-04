use crate::bbox::BBox;
use crate::policy::{Policy, PolicyInto};
use crate::tarpc::context::TahiniContext;

pub trait TahiniTransform<TargetPolicy: Policy, TargetType> {
    fn transform_policy(self, context: &TahiniContext) -> Result<TargetType, String>;
}

impl<
        T: 'static,
        SourcePolicy: Policy + 'static + PolicyInto<TargetPolicy>,
        TargetPolicy: Policy + 'static,
    > TahiniTransform<TargetPolicy, BBox<T, TargetPolicy>> for BBox<T, SourcePolicy>
{
    fn transform_policy(self, context: &TahiniContext) -> Result<BBox<T, TargetPolicy>, String> {
        let (t, p) = self.consume();
        Ok(BBox::new(t, p.into_policy(context)?))
    }
}

impl<
        TargetType: 'static,
        TargetPolicy: Policy,
        SourceType: TahiniTransform<TargetPolicy, TargetType>,
    > TahiniTransform<TargetPolicy, Vec<TargetType>> for Vec<SourceType>
{
    fn transform_policy(self, context: &TahiniContext) -> Result<Vec<TargetType>, String> {
        self.into_iter()
            .map(|source_elem: SourceType| source_elem.transform_policy(context))
            .collect::<Result<Vec<_>, String>>()
    }
}

impl<
        TargetType: 'static,
        TargetPolicy: Policy,
        SourceType: TahiniTransform<TargetPolicy, TargetType>,
        E: std::error::Error,
    > TahiniTransform<TargetPolicy, Result<TargetType, E>> for Result<SourceType, E>
{
    fn transform_policy(self, context: &TahiniContext) -> Result<Result<TargetType, E>, String> {
        match self {
            Ok(inner) => Ok(Ok(inner.transform_policy(context)?)),
            Err(e) => Ok(Err(e)),
        }
    }
}

impl<
        TargetType: 'static,
        TargetPolicy: Policy,
        SourceType: TahiniTransform<TargetPolicy, TargetType>,
    > TahiniTransform<TargetPolicy, Option<TargetType>> for Option<SourceType>
{
    fn transform_policy(self, context: &TahiniContext) -> Result<Option<TargetType>, String> {
        self.map(|some| some.transform_policy(context)).transpose()
    }
}
