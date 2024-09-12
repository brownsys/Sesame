use std::fmt::{Debug, Display, Formatter, Write};
use crate::bbox::BBox;
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicy, FrontendPolicy, Policy, Reason, RefPolicy, SchemaPolicy};

// TestPolicy<P> is the same as P, except it also allows direct access to boxed data for testing
// purposes.
#[derive(Clone)]
pub struct TestPolicy<P: Policy + 'static> {
    p: P,
}

impl<P: Policy + 'static + Debug> Debug for TestPolicy<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestPolicy({:?})", self.p)
    }
}

impl<P: Policy + 'static> TestPolicy<P> {
    pub fn new(p: P) -> Self {
        Self { p }
    }
    pub fn policy(&self) -> &P {
        &self.p
    }
}
impl<P: Policy + 'static> Policy for TestPolicy<P> {
    fn name(&self) -> String { format!("TestPolicy<{}>", self.p.name()) }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.p.check(context, reason)
    }
    default fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        Ok(AnyPolicy::new(TestPolicy::new(self.p.join(other)?)))
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        Ok(TestPolicy::new(self.p.join_logic(other.p)?))
    }
}
impl<P: Policy + Clone + 'static> Policy for TestPolicy<P> {
    default fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<TestPolicy<P>>() {
            let other = other.specialize::<TestPolicy<P>>().unwrap().p;
            Ok(AnyPolicy::new(TestPolicy::new(self.p.join_logic(other)?)))
        } else {
            Ok(AnyPolicy::new(TestPolicy::new(self.p.join(other)?)))
        }
    }
}

impl<P: Policy + SchemaPolicy + 'static> SchemaPolicy for TestPolicy<P> {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self {
        TestPolicy { p: P::from_row(table_name, row) }
    }
}
impl<P: Policy + FrontendPolicy + 'static> FrontendPolicy for TestPolicy<P> {
    fn from_request(request: &rocket::Request<'_>) -> Self {
        TestPolicy { p: P::from_request(request) }
    }
    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>) -> Self {
        Self { p: P::from_cookie(name, cookie, request) }
    }
}

// Can construct a test policy from the underlying policy directly.
impl<P: Policy + 'static> From<P> for TestPolicy<P> {
    fn from(value: P) -> Self {
        TestPolicy::new(value)
    }
}

// Test policy can be discarded.
impl<T, P: 'static + Policy> BBox<T, TestPolicy<P>> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}

// Test policy allows testing equality.
impl<T: PartialEq, P: Policy + 'static> PartialEq for BBox<T, TestPolicy<P>> {
    fn eq(&self, other: &Self) -> bool {
        self.data().eq(other.data())
    }
}
impl<T: Eq + PartialEq, P: Policy + 'static> Eq for BBox<T, TestPolicy<P>> {}

// Test Policy allows debugging.
impl<T: Debug, P: Policy + 'static> Debug for BBox<T, TestPolicy<P>> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BBox({:?}, <{}>)", self.data(), self.policy().name())
    }
}

impl<T: Debug, P: Debug + Policy + 'static> Debug for BBox<T, TestPolicy<P>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BBox({:?}, {:?})", self.data(), self.policy())
    }
}

// Same but for RefPolicy<TestPolicy>
impl<'a, T, P: 'static + Policy> BBox<T, RefPolicy<'a, TestPolicy<P>>> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}
impl<'a, T: PartialEq, P: Policy + 'static> PartialEq for BBox<T, RefPolicy<'a, TestPolicy<P>>> {
    fn eq(&self, other: &Self) -> bool {
        self.data().eq(other.data())
    }
}
impl<'a, T: Eq + PartialEq, P: Policy + 'static> Eq for BBox<T, RefPolicy<'a, TestPolicy<P>>> {}
impl<'a, T: Debug, P: Policy + 'static> Debug for BBox<T, RefPolicy<'a, TestPolicy<P>>> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BBox({:?}, <{}>)", self.data(), self.policy().name())
    }
}
impl<'a, T: Debug, P: Debug + Policy + 'static> Debug for BBox<T, RefPolicy<'a, TestPolicy<P>>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BBox({:?}, {:?})", self.data(), self.policy().policy())
    }
}