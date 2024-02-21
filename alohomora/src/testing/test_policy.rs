use std::any::Any;
use std::fmt::{Debug, Formatter, Write};
use crate::bbox::BBox;
use crate::policy::{AnyPolicy, FrontendPolicy, Policy, SchemaPolicy};
use crate::rocket::BBoxRequest;

// TestPolicy<P> is the same as P, except it also allows direct access to boxed data for testing
// purposes.
#[derive(Clone, PartialEq, Eq, Debug)]
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

// Test policy can be discarded, logged, etc
impl<T, P: 'static + Policy + Clone> BBox<T, TestPolicy<P>> {
    pub fn discard_box(self) -> T {
        self.t
    }
}
impl<T: Debug, P: 'static + Policy + Clone> Debug for BBox<T, TestPolicy<P>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Box(")?;
        self.t.fmt(f)?;
        f.write_char(')')
    }
}
impl<T: PartialEq, P: 'static + Policy + PartialEq + Clone> PartialEq for BBox<T, TestPolicy<P>> {
    fn eq(&self, other: &Self) -> bool {
        self.t.eq(&other.t)
    }
}
impl<T: PartialEq + Eq, P: 'static + Policy + PartialEq + Eq + Clone> Eq for BBox<T, TestPolicy<P>> {}
