use crate::bbox::BBox;
use crate::policy::AnyPolicy;
use crate::r#type::{AlohomoraType, fold};

pub fn sandbox<S: AlohomoraType, R, F: FnOnce(S::Out) -> R>(
    s: S,
    lambda: F,
) -> BBox<R, AnyPolicy> {
    let outer_boxed = fold(s).unwrap();
    BBox::new(lambda(outer_boxed.t), outer_boxed.p)
}