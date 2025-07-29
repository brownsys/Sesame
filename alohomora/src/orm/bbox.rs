use crate::bbox::BBox;
use crate::policy::Policy;
use std::fmt::{Debug, Formatter};

impl<T: Debug, P: Policy + Debug> Debug for BBox<T, P> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BBox")
            .field("data", self.data())
            .field("policy", &self.policy())
            .finish()
    }
}

impl<T: PartialEq, P: Policy + PartialEq> PartialEq for BBox<T, P> {
    default fn eq(&self, other: &Self) -> bool {
        self.data() == other.data() && self.policy() == other.policy()
    }
}

impl<T: PartialEq + Eq, P: Policy + PartialEq + Eq> Eq for BBox<T, P> {}

