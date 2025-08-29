use crate::context::UnprotectedContext;
use crate::pcon::PCon;
use crate::policy::{Policy, Reason};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use serde::ser::SerializeStruct;

// NoPolicy can be directly discarded.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NoPolicy {}

impl NoPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Policy for NoPolicy {
    fn name(&self) -> String {
        String::from("NoPolicy")
    }
    fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        true
    }
}

impl Default for NoPolicy {
    fn default() -> Self {
        NoPolicy {}
    }
}

// NoPolicy can be discarded, logged, etc
impl<T> PCon<T, NoPolicy> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}
impl<T: Debug> Debug for PCon<T, NoPolicy> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PCon")
            .field("data", self.data())
            .field("policy", &"NoPolicy")
            .finish()
    }
}
impl<T: PartialEq> PartialEq for PCon<T, NoPolicy> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}


impl<T: Serialize> Serialize for BBox<T, NoPolicy> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bbox_ser = serializer.serialize_struct("BBox", 2)?;
        bbox_ser.serialize_field("fb", self.data())?;
        bbox_ser.serialize_field("p", self.policy())?;
        bbox_ser.end()
    }
}
