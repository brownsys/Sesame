use crate::context::UnprotectedContext;
use crate::pcon::PCon;
use crate::policy::{OptionPolicy, Policy, Reason, RefPolicy};

use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

// TestPolicy<P> is the same as P, except it also allows direct access to boxed data for testing
// purposes.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestPolicy<P: Policy> {
    p: P,
}

impl<P: Policy> TestPolicy<P> {
    pub fn new(p: P) -> Self {
        Self { p }
    }
    pub fn policy(&self) -> &P {
        &self.p
    }
    pub fn mut_policy(&mut self) -> &mut P {
        &mut self.p
    }
    pub fn into_inner(self) -> P {
        self.p
    }
}

impl<P: Policy> Policy for TestPolicy<P> {
    fn name(&self) -> String {
        format!("TestPolicy<{}>", self.p.name())
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        self.p.check(context, reason)
    }
}

impl<P: Policy> From<P> for TestPolicy<P> {
    fn from(value: P) -> Self {
        TestPolicy::new(value)
    }
}

// Test policy can be discarded, logged, etc
impl<T, P: Policy> PCon<T, TestPolicy<P>> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}

impl<T: Debug, P: Policy + Debug> Debug for PCon<T, TestPolicy<P>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PCon")
            .field("data", self.data())
            .field("policy", self.policy())
            .finish()
    }
}
impl<T: PartialEq, P: Policy + PartialEq> PartialEq for PCon<T, TestPolicy<P>> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data() && self.policy() == other.policy()
    }
}

impl<T: Eq, P: Policy + Eq> Eq for PCon<T, TestPolicy<P>> {}

// Same but for RefPolicy<TestPolicy>
impl<'a, T, P: Policy> PCon<&'a T, RefPolicy<'a, TestPolicy<P>>> {
    pub fn discard_box(self) -> &'a T {
        self.consume().0
    }
}
impl<'a, T: Debug, P: Policy + Debug> Debug for PCon<&'a T, RefPolicy<'a, TestPolicy<P>>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PCon")
            .field("data", self.data())
            .field("policy", self.policy())
            .finish()
    }
}
impl<'a, T: PartialEq, P: Policy + PartialEq> PartialEq
    for PCon<&'a T, RefPolicy<'a, TestPolicy<P>>>
{
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data() && self.policy() == other.policy()
    }
}
impl<'a, T: Eq, P: Policy + Eq> Eq for PCon<&'a T, RefPolicy<'a, TestPolicy<P>>> {}

// Same but for OptionPolicy<TestPolicy>
impl<T, P: Policy> PCon<T, OptionPolicy<TestPolicy<P>>> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }
}
impl<T: Debug, P: Policy + Debug> Debug for PCon<T, OptionPolicy<TestPolicy<P>>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PCon")
            .field("data", self.data())
            .field("policy", self.policy())
            .finish()
    }
}
impl<T: PartialEq, P: Policy + PartialEq> PartialEq for PCon<T, OptionPolicy<TestPolicy<P>>> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data() && self.policy() == other.policy()
    }
}
impl<T: Eq, P: Policy + Eq> Eq for PCon<T, OptionPolicy<TestPolicy<P>>> {}
