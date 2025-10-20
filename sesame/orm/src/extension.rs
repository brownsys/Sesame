use crate::ORMPolicy;
use sesame::bbox::BBox;
use sesame::extensions::SesameExtension;

#[derive(Clone)]
pub struct ORMBBox<T, P: ORMPolicy> {
    pub(crate) t: T,
    pub(crate) p: P,
}

struct ORMExtension {}

impl<T, P: ORMPolicy> SesameExtension<T, P, ORMBBox<T, P>> for ORMExtension {
    fn apply(self, data: T, policy: P) -> ORMBBox<T, P> {
        ORMBBox { t: data, p: policy }
    }
    fn apply_ref(self, _data: &T, _policy: &P) -> ORMBBox<T, P> {
        panic!("");
    }
}

impl<T, P: ORMPolicy> From<BBox<T, P>> for ORMBBox<T, P> {
    fn from(bbox: BBox<T, P>) -> Self {
        bbox.apply_extension(ORMExtension {})
    }
}
impl<T, P: ORMPolicy> Into<BBox<T, P>> for ORMBBox<T, P> {
    fn into(self) -> BBox<T, P> {
        BBox::new(self.t, self.p)
    }
}
