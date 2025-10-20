// BBox
use sesame::bbox::{BBox, EitherBBox};
use sesame::policy::{AnyPolicy, AnyPolicyable};

// Our params may be boxed or clear.
pub trait BBoxParam {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy>;
}

// Implement for basic types.
macro_rules! bbox_param_impl {
  ($($T:ty,)+) => (
    $(
    impl BBoxParam for $T {
        fn get(self) -> EitherBBox<mysql::Value, AnyPolicy> {
            EitherBBox::Left(self.into())
        }
    }
    )+
  );
}
bbox_param_impl!(String, &str,);
bbox_param_impl!(u8, u16, u32, u64, u128, usize,);
bbox_param_impl!(i8, i16, i32, i64, i128, isize,);
bbox_param_impl!(bool, f32, f64,);

impl<T: Into<mysql::Value>, P: AnyPolicyable> BBoxParam for BBox<T, P> {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy> {
        EitherBBox::Right(self.into_any_policy_no_clone().into_bbox())
    }
}

/*
impl<T: Into<mysql::Value>> BBoxParam for T where T: From<mysql::Value> {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy> {
        EitherBBox::<mysql::Value, AnyPolicy>::Value(self.into())
    }
}
 */

impl<T: Into<mysql::Value>, P: AnyPolicyable> BBoxParam for EitherBBox<T, P> {
    fn get(self) -> EitherBBox<mysql::Value, AnyPolicy> {
        match self {
            EitherBBox::Left(t) => EitherBBox::Left(t.into()),
            EitherBBox::Right(bbox) => {
                EitherBBox::Right(bbox.into_any_policy_no_clone().into_bbox())
            }
        }
    }
}
