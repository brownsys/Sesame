use mysql::Value;
use std::any::{Any, TypeId};

// Public facing Policy traits.
pub trait Policy {
    fn name(&self) -> String;
    fn check(&self, context: &dyn Any) -> bool;
}
pub trait SchemaPolicy: Policy {
    fn from_row(row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized;
}

pub trait FrontendPolicy: Policy {
    fn from_request() -> Self
    where
        Self: Sized;
}

// Any (owned) Policy.
trait TypeIdPolicyTrait: Policy + Any {}
impl<P: Policy + 'static> TypeIdPolicyTrait for P {}

pub struct AnyPolicy {
    policy: Box<dyn TypeIdPolicyTrait>,
}
impl AnyPolicy {
    pub fn new<P: Policy + 'static>(p: P) -> Self {
        Self {
            policy: Box::new(p),
        }
    }
    pub fn is<P: Policy + 'static>(&self) -> bool {
        TypeId::of::<P>() == self.policy.as_ref().type_id()
    }
    pub fn specialize<P: Policy + 'static>(self) -> Result<P, String> {
        if self.is::<P>() {
            let raw = Box::into_raw(self.policy);
            let raw = raw as *mut P;
            Ok(*unsafe { Box::from_raw(raw) })
        } else {
            Err(format!(
                "Cannot convert '{}' to '{:?}'",
                self.name(),
                TypeId::of::<P>()
            ))
        }
    }
}
impl Policy for AnyPolicy {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.policy.check(context)
    }
}

// NoPolicy can be directly discarded.
#[derive(Clone)]
pub struct NoPolicy {}
impl Policy for NoPolicy {
    fn name(&self) -> String {
        String::from("NoPolicy")
    }
    fn check(&self, _context: &dyn Any) -> bool {
        true
    }
}

// Allows combining policies with AND
#[derive(Clone)]
pub struct PolicyAnd<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyAnd<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
}
impl<P1: Policy, P2: Policy> Policy for PolicyAnd<P1, P2> {
    fn name(&self) -> String {
        format!("({} AND {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.p1.check(context) && self.p2.check(context)
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyAnd<P1, P2> {
    fn from_row(row: &Vec<Value>) -> Self {
        Self {
            p1: P1::from_row(row),
            p2: P2::from_row(row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyAnd<P1, P2> {
    fn from_request() -> Self {
        Self {
            p1: P1::from_request(),
            p2: P2::from_request(),
        }
    }
}

#[derive(Clone)]
pub struct PolicyOr<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyOr<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
}
impl<P1: Policy, P2: Policy> Policy for PolicyOr<P1, P2> {
    fn name(&self) -> String {
        format!("({} OR {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.p1.check(context) || self.p2.check(context)
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyOr<P1, P2> {
    fn from_row(row: &Vec<Value>) -> Self {
        Self {
            p1: P1::from_row(row),
            p2: P2::from_row(row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyOr<P1, P2> {
    fn from_request() -> Self {
        Self {
            p1: P1::from_request(),
            p2: P2::from_request(),
        }
    }
}
