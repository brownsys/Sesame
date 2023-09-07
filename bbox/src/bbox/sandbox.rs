use crate::bbox::BBox;
use crate::policy::{NoPolicy, Policy};

// BBox and containers of it are sandboxable.
pub trait Sandboxable {
    type Out;
    fn unbox(self) -> Self::Out;
}
impl<T, P: Policy> Sandboxable for BBox<T, P> {
    type Out = T;
    fn unbox(self) -> Self::Out {
        self.into_temporary_unbox()
    }
}
impl<'a, T, P: Policy> Sandboxable for &'a BBox<T, P> {
    type Out = &'a T;
    fn unbox(self) -> Self::Out {
        self.temporary_unbox()
    }
}
impl<S: Sandboxable> Sandboxable for Vec<S> {
    type Out = Vec<S::Out>;
    fn unbox(self) -> Self::Out {
        self.into_iter().map(|s| s.unbox()).collect()
    }
}
impl<'a, S> Sandboxable for &'a Vec<S>
where
    &'a S: Sandboxable,
{
    type Out = Vec<<&'a S as Sandboxable>::Out>;
    fn unbox(self) -> Self::Out {
        self.iter().map(|s| s.unbox()).collect()
    }
}

// TODO(artem): merge policies together or use lambda to acquire policy.
// Sandbox execute with a container of bboxes.
pub fn sandbox_execute<S: Sandboxable, R, F: FnOnce(S::Out) -> R>(
    s: S,
    lambda: F,
) -> BBox<R, NoPolicy> {
    BBox::new(lambda(s.unbox()), NoPolicy {})
}

// Need to generalize this to many arguments.
pub fn sandbox_combine<S1: Sandboxable, S2: Sandboxable, R, F: FnOnce(S1::Out, S2::Out) -> R>(
    s1: S1,
    s2: S2,
    lambda: F,
) -> BBox<R, NoPolicy> {
    BBox::new(lambda(s1.unbox(), s2.unbox()), NoPolicy {})
}
