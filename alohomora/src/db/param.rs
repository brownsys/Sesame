// BBox
use crate::bbox::{BBox, EitherBBox};
use crate::policy::{AnyPolicy, AnyPolicyClone, AnyPolicyCloneDyn, AnyPolicyable};

// Our params may be boxed or clear.
pub trait BBoxParam {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicyClone>;
}

impl<T: Into<mysql::Value>, P: AnyPolicyCloneDyn> BBoxParam for BBox<T, P> {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicyClone> {
        EitherBBox::BBox(self.into_any_policy().into_bbox())
    }
}

impl<T: Into<mysql::Value>> BBoxParam for T {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicyClone> {
        EitherBBox::<mysql::Value, AnyPolicyClone>::Value(self.into())
    }
}

impl<T: Into<mysql::Value>, P: AnyPolicyCloneDyn> BBoxParam for EitherBBox<T, P> {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicyClone> {
        match self {
            EitherBBox::Value(t) => EitherBBox::Value(t.into()),
            EitherBBox::BBox(bbox) => EitherBBox::BBox(bbox.into_any_policy().into_bbox()),
        }
    }
}
