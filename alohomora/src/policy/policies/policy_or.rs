use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicy, FrontendPolicy, Policy, Reason, SchemaPolicy};

#[derive(Clone)]
pub struct PolicyOr<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyOr<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
    pub fn policy1(&self) -> &P1 { &self.p1 }
    pub fn policy2(&self) -> &P2 { &self.p2 }
}
impl<P1: Policy, P2: Policy> Policy for PolicyOr<P1, P2> {
    fn name(&self) -> String {
        format!("({} OR {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.p1.check(context, reason.clone()) || self.p2.check(context, reason)
    }
    // TODO(babman): find ways to make join work under PolicyOr
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> {
        todo!()
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyOr<P1, P2> {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self {
        Self {
            p1: P1::from_row(table_name, row),
            p2: P2::from_row(table_name, row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyOr<P1, P2> {
    fn from_request(request: &rocket::Request<'_>) -> Self {
        Self {
            p1: P1::from_request(request),
            p2: P2::from_request(request),
        }
    }
    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>) -> Self {
        Self {
            p1: P1::from_cookie(name, cookie, request),
            p2: P2::from_cookie(name, cookie, request),
        }
    }
}