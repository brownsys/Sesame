use std::any::Any;
use crate::policy::{AnyPolicy, FrontendPolicy, Policy, SchemaPolicy};
use crate::rocket::BBoxRequest;

// TestPolicy<P> is the same as P, except it also allows direct access to boxed data for testing
// purposes.
#[derive(Clone, PartialEq, Eq)]
pub struct TestPolicy<P: 'static + Policy + Clone> {
    p: P,
}

impl<P: 'static + Policy + Clone> TestPolicy<P> {
    pub fn new(p: P) -> Self {
        Self { p }
    }
    pub fn policy(&self) -> &P {
        &self.p
    }
}
impl<P: 'static + Policy + Clone> Policy for TestPolicy<P> {
    fn name(&self) -> String { format!("TestPolicy<{}>", self.p.name()) }
    fn check(&self, context: &dyn Any) -> bool {
        self.p.check(context)
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<TestPolicy<P>>() {
            let other = other.specialize::<TestPolicy<P>>().unwrap();
            Ok(AnyPolicy::new(self.p.join_logic(other.p)?))
        } else {
            Ok(AnyPolicy::new(TestPolicy { p: self.p.join(other)? }))
        }
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        Ok(TestPolicy { p: self.p.join_logic(other.p)? })
    }
}

impl<P: 'static + Policy + SchemaPolicy + Clone> SchemaPolicy for TestPolicy<P> {
    fn from_row(row: &Vec<mysql::Value>) -> Self {
        TestPolicy { p: P::from_row(row) }
    }
}
impl<P: 'static + Policy + FrontendPolicy + Clone> FrontendPolicy for TestPolicy<P> {
    fn from_request<'a, 'r>(request: &'a BBoxRequest<'a, 'r>) -> Self {
        TestPolicy { p: P::from_request(request) }
    }
    fn from_cookie() -> Self {
        TestPolicy { p: P::from_cookie() }
    }
}

impl<P: 'static + Policy + Clone> From<P> for TestPolicy<P> {
    fn from(value: P) -> Self {
        TestPolicy::new(value)
    }
}