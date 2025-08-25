use std::convert::Infallible;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::policy::{AnyPolicyDyn, AsLeaf, AsNoReflection, IsNoPolicy, NameVisitor, NormalizeVisitor, Policy};

// B: &'r mut dyn AnyPolicyTrait.
// B: Box<dyn AnyPolicyTrait>
pub enum PolicyReflection<'a, L: AsLeaf, NR: AsNoReflection<'a>> {
    NoReflection(NR), // Something we cannot use reflection on, e.g. a type that is not std::any::Any.
    Leaf(L),
    PolicyAnd(Box<Self>, Box<Self>),
    PolicyOr(Box<Self>, Box<Self>),
    PolicyRef(NR, Box<RefReflection<'a>>),
    OptionPolicy(Option<Box<Self>>),
    AnyPolicy(Box<Self>),
    TestPolicy(Box<Self>),
    _Unreachable(Infallible, PhantomData<&'a ()>),
}

// Useful Aliases.
pub type OwnedReflection<'a> = PolicyReflection<'a, Box<dyn AnyPolicyDyn>, Box<dyn Policy + 'a>>;
pub type MutRefReflection<'a> =
    PolicyReflection<'a, &'a mut (dyn AnyPolicyDyn), &'a mut (dyn Policy + 'a)>;
pub type RefReflection<'a> = PolicyReflection<'a, &'a dyn AnyPolicyDyn, &'a (dyn Policy + 'a)>;

// Normalize a reflection enum (by removing AnyPolicies and TestPolicies)
impl<'a, L: AsLeaf, NR: AsNoReflection<'a>> PolicyReflection<'a, L, NR> {
    // Check if this is in effect a no policy.
    pub fn is_no_policy(&'a self) -> bool {
        let mut v = IsNoPolicy {};
        self.postfix_visit_by_ref(&mut v)
    }

    // Normalize representation by removing AnyPolicies.
    pub fn normalize(self) -> Self {
        let mut v = NormalizeVisitor {};
        self.postfix_visit_by_move(&mut v)
    }
}

impl<'a, L: AsLeaf + 'a, NR: AsNoReflection<'a> + 'a> Debug for PolicyReflection<'a, L, NR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = NameVisitor {};
        f.write_str(&self.postfix_visit_by_ref(&mut v))
    }
}