use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;


pub fn execute_sandbox<S: AlohomoraType, R, F: FnOnce(S::Out) -> R>(
    s: S,
    lambda: F,
) -> BBox<R, AnyPolicy> {
    let outer_boxed = fold(s).unwrap();
    let (t, p) = outer_boxed.consume();
    BBox::new(lambda(t), p)
}