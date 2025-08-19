use crate::bbox::BBox;
use crate::context::UnprotectedContext;
use crate::policy::{FrontendPolicy, Policy, Reason, ReflexiveJoin, SchemaPolicy};
use std::fmt::{Debug, Formatter};
use serde::Serialize;

// NoPolicy can be directly discarded.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
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

    /*
    // Join.
    fn policy_type_enum(&mut self) -> PolicyTypeEnum<'_> {
        PolicyTypeEnum::Leaf(self)
    }
    fn can_join_with(&mut self, p: &PolicyTypeEnum<'_>) -> bool {
        match p {
            PolicyTypeEnum::Leaf(s) => s.upcast_any().is::<Self>(),
            _ => false,
        }
    }
    fn join(&mut self, p: PolicyTypeEnum<'_>) -> bool {
        self.can_join_with(&p)
    }
     */
}

// Guarantees that we can join NoPolicies together.
impl ReflexiveJoin for NoPolicy {
    fn reflexive_join(&mut self, _other: &mut Self) {}
}

// Can be used for schema and frontend (rocket) policy associations.
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
