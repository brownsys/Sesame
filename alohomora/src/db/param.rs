// BBox
use crate::bbox::{BBox, EitherBBox};
use crate::policy::{AnyPolicy, AnyPolicyable};

// Our params may be boxed or clear.
pub trait BBoxParam {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy>;
}

impl<T: Into<mysql::Value>, P: AnyPolicyable> BBoxParam for BBox<T, P> {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy> {
        EitherBBox::BBox(self.into_any_policy_no_clone().into_bbox())
    }
}

impl<T: Into<mysql::Value>> BBoxParam for T {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy> {
        EitherBBox::<mysql::Value, AnyPolicy>::Value(self.into())
    }
}

impl<T: Into<mysql::Value>, P: AnyPolicyable> BBoxParam for EitherBBox<T, P> {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy> {
        match self {
            EitherBBox::Value(t) => EitherBBox::Value(t.into()),
            EitherBBox::BBox(bbox) => EitherBBox::BBox(bbox.into_any_policy_no_clone().into_bbox()),
        }
    }
}
