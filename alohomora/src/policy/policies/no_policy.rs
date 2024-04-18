use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicy, FrontendPolicy, Policy, Reason, SchemaPolicy};

// NoPolicy can be directly discarded.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NoPolicy {}
impl NoPolicy {
    pub fn new () -> Self {
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
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
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
        _request: &'a rocket::Request<'r>) -> Self { Self {}}
}

impl Default for NoPolicy {
    fn default() -> Self {
        NoPolicy {}
    }
}