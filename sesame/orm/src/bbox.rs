use std::fmt::{Debug, Formatter};

use crate::{ORMBBox, ORMPolicy};

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
