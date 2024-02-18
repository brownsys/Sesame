// BBox
use crate::bbox::{BBox, EitherBBox};
use crate::policy::{AnyPolicy, Policy};

// Our params may be boxed or clear.
#[derive(Clone)]
pub struct BBoxParam(pub(super) EitherBBox<mysql::Value, AnyPolicy>);

// Auto convert mysql::Value and bbox to Value.
impl<T: Into<mysql::Value>> From<T> for BBoxParam {
    fn from(x: T) -> BBoxParam {
        BBoxParam(EitherBBox::Value(x.into()))
    }
}
impl<T: Into<mysql::Value>, P: Policy + Clone + 'static> From<BBox<T, P>> for BBoxParam {
    fn from(x: BBox<T, P>) -> BBoxParam {
        BBoxParam(EitherBBox::BBox(x.into_bbox().any_policy()))
    }
}