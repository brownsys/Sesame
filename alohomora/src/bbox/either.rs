use crate::bbox::BBox;
use crate::policy::Policy;

// A type that contains either T or BBox<T>.
pub enum EitherBBox<T, P: Policy> {
    Value(T),
    BBox(BBox<T, P>),
}

// EitherBBox is cloneable if T is cloneable.
impl<T: Clone, P: Policy + Clone> Clone for EitherBBox<T, P> {
    fn clone(&self) -> Self {
        match self {
            EitherBBox::Value(value) => EitherBBox::Value(value.clone()),
            EitherBBox::BBox(bbox) => EitherBBox::BBox(bbox.clone()),
        }
    }
}

// Can be constructed from either value or BBox.
impl<T, P: Policy> From<T> for EitherBBox<T, P> {
    fn from(x: T) -> EitherBBox<T, P> {
        EitherBBox::<T, P>::Value(x)
    }
}
impl<T, P: Policy> From<BBox<T, P>> for EitherBBox<T, P> {
    fn from(x: BBox<T, P>) -> EitherBBox<T, P> {
        EitherBBox::BBox(x)
    }
}
