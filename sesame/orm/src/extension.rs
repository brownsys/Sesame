use crate::ORMPolicy;
use sesame::bbox::BBox;
use sesame::extensions::{SesameExtension, UncheckedSesameExtension};
use std::fmt::{Debug, Formatter};

// sesame_orm's version of a BBox.
#[derive(Clone)]
pub struct ORMBBox<T, P: ORMPolicy> {
    pub(crate) t: T,
    pub(crate) p: P,
}

// ORMBBox is leaky: should improve the below later.
impl<T: Debug, P: ORMPolicy + Debug> Debug for ORMBBox<T, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ORMBBox")
            .field("data", &self.t)
            .field("policy", &self.p)
            .finish()
    }
}
impl<T: PartialEq, P: ORMPolicy + PartialEq> PartialEq for ORMBBox<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.p == other.p
    }
}
impl<T: PartialEq + Eq, P: ORMPolicy + PartialEq + Eq> Eq for ORMBBox<T, P> {}

// We use a Sesame extension to transform BBox into an ORMBBox.
struct ORMExtension {}
impl UncheckedSesameExtension for ORMExtension {}
impl<T, P: ORMPolicy> SesameExtension<T, P, ORMBBox<T, P>> for ORMExtension {
    fn apply(&mut self, data: T, policy: P) -> ORMBBox<T, P> {
        ORMBBox { t: data, p: policy }
    }
}

// Conversions from and to ORMBBox.
impl<T, P: ORMPolicy> From<BBox<T, P>> for ORMBBox<T, P> {
    fn from(bbox: BBox<T, P>) -> Self {
        bbox.unchecked_extension(&mut ORMExtension {})
    }
}
impl<T, P: ORMPolicy> Into<BBox<T, P>> for ORMBBox<T, P> {
    fn into(self) -> BBox<T, P> {
        BBox::new(self.t, self.p)
    }
}
