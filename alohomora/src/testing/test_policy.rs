use crate::bbox::BBox;
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyBB, AnyPolicyClone, AnyPolicyTrait, FrontendPolicy, Policy, Reason, RefPolicy, SchemaPolicy};
use std::fmt::{Debug, Formatter};

// TestPolicy<P> is the same as P, except it also allows direct access to boxed data for testing
// purposes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestPolicy<P: AnyPolicyTrait> {
    p: P,
}

impl<P: AnyPolicyTrait> TestPolicy<P> {
    pub fn new(p: P) -> Self {
        Self { p }
    }
    pub fn policy(&self) -> &P {
        &self.p
    }
}
impl<P: AnyPolicyTrait> Policy for TestPolicy<P> {
    fn name(&self) -> String {
        format!("TestPolicy<{}>", self.p.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.p.check(context, reason)
    }
    fn join(&self, other: AnyPolicyBB) -> Result<AnyPolicyBB, ()> {
        if other.is::<TestPolicy<P>>() {
            let other = other.specialize::<TestPolicy<P>>().unwrap();
            Ok(AnyPolicyBB::new(self.join_logic(other)?))
        } else {
            Ok(AnyPolicyBB::new(TestPolicy::new(self.p.join(other)?)))
        }
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        Ok(TestPolicy::new(self.p.join_logic(other.p)?))
    }
}

impl<P: AnyPolicyTrait + SchemaPolicy> SchemaPolicy for TestPolicy<P> {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self {
        TestPolicy {
            p: P::from_row(table_name, row),
        }
    }
}
impl<P: AnyPolicyTrait + FrontendPolicy> FrontendPolicy for TestPolicy<P> {
    fn from_request(request: &rocket::Request<'_>) -> Self {
        TestPolicy {
            p: P::from_request(request),
        }
    }
    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>,
    ) -> Self {
        Self {
            p: P::from_cookie(name, cookie, request),
        }
    }
}

impl<P: AnyPolicyTrait> From<P> for TestPolicy<P> {
    fn from(value: P) -> Self {
        TestPolicy::new(value)
    }
}

// Test policy can be discarded, logged, etc
impl<T, P: 'static + Policy + Clone> BBox<T, TestPolicy<P>> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}

impl<T: Debug, P: AnyPolicyTrait + Debug> Debug for BBox<T, TestPolicy<P>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BBox")
            .field("data", self.data())
            .field("policy", self.policy())
            .finish()
    }
}
impl<T: PartialEq, P: AnyPolicyTrait + PartialEq> PartialEq for BBox<T, TestPolicy<P>> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data() && self.policy() == other.policy()
    }
}

impl<T: Eq, P: AnyPolicyTrait + Eq> Eq for BBox<T, TestPolicy<P>> {}

// Same but for RefPolicy<TestPolicy>
impl<'a, T, P: AnyPolicyTrait> BBox<&'a T, RefPolicy<'a, TestPolicy<P>>> {
    pub fn discard_box(self) -> &'a T {
        self.consume().0
    }
}
impl<'a, T: Debug, P: AnyPolicyTrait + Debug> Debug for BBox<&'a T, RefPolicy<'a, TestPolicy<P>>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BBox")
            .field("data", self.data())
            .field("policy", self.policy())
            .finish()
    }
}
impl<'a, T: PartialEq, P: AnyPolicyTrait + PartialEq> PartialEq for BBox<&'a T, RefPolicy<'a, TestPolicy<P>>> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data() && self.policy() == other.policy()
    }
}
impl<'a, T: Eq, P: AnyPolicyTrait + Eq> Eq for BBox<&'a T, RefPolicy<'a, TestPolicy<P>>> {}
