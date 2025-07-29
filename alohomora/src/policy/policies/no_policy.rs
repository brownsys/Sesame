use crate::bbox::BBox;
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyBB, FrontendPolicy, Policy, Reason, SchemaPolicy};
use std::fmt::{Debug, Formatter};

// NoPolicy can be directly discarded.
#[derive(Clone, PartialEq, Eq, Debug)]
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
    fn join(&self, other: AnyPolicyBB) -> Result<AnyPolicyBB, ()> {
        Ok(other)
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        Ok(other)
    }
}

impl SchemaPolicy for NoPolicy {
    fn from_row(_table_name: &str, _row: &Vec<mysql::Value>) -> Self {
        Self {}
    }
}
impl FrontendPolicy for NoPolicy {
    fn from_request(_request: &rocket::Request<'_>) -> Self {
        Self {}
    }

    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a rocket::http::Cookie<'static>,
        _request: &'a rocket::Request<'r>,
    ) -> Self {
        Self {}
    }
}

impl Default for NoPolicy {
    fn default() -> Self {
        NoPolicy {}
    }
}

// NoPolicy can be discarded, logged, etc
impl<T> BBox<T, NoPolicy> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}
impl<T: Debug> Debug for BBox<T, NoPolicy> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BBox").field("data", self.data()).finish()
    }
}
impl<T: PartialEq> PartialEq for BBox<T, NoPolicy> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}
