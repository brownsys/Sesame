use serde::Serialize;

use crate::context::UnprotectedContext;
use crate::policy::{Policy, Reason};

#[derive(Clone, Serialize, PartialEq, Eq, Debug)]
pub struct PolicyAnd<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyAnd<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
    pub fn policy1(&self) -> &P1 {
        &self.p1
    }
    pub fn policy2(&self) -> &P2 {
        &self.p2
    }
    pub fn policies(&self) -> (&P1, &P2) {
        (&self.p1, &self.p2)
    }
    pub fn mut_policies(&mut self) -> (&mut P1, &mut P2) {
        (&mut self.p1, &mut self.p2)
    }
    pub fn into_inner(self) -> (P1, P2) {
        (self.p1, self.p2)
    }
}

impl<P1: Policy, P2: Policy> Policy for PolicyAnd<P1, P2> {
    fn name(&self) -> String {
        format!("PolicyAnd({} AND {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.p1.check(context, reason.clone()) && self.p2.check(context, reason)
    }
}
