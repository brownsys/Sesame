use crate::bbox::BBox;

// A type that contains either T or BBox<T>.
pub enum EitherBBox<T> {
    Value(T),
    BBox(BBox<T>),
}

// EitherBBox is clonable if T is clonable.
impl<T: Clone> Clone for EitherBBox<T> {
    fn clone(&self) -> Self {
        match self {
            EitherBBox::Value(value) => EitherBBox::Value(value.clone()),
            EitherBBox::BBox(bbox) => EitherBBox::BBox(bbox.clone()),
        }
    }
}

// Can be constructed from either value or BBox.
impl<T> From<T> for EitherBBox<T> {
    fn from(x: T) -> EitherBBox<T> {
        EitherBBox::Value(x)
    }
}
impl<T> From<BBox<T>> for EitherBBox<T> {
    fn from(x: BBox<T>) -> EitherBBox<T> {
        EitherBBox::BBox(x)
    }
}
