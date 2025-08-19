use std::convert::Infallible;
use std::marker::PhantomData;
use std::ops::{DerefMut};
use crate::fold_in::NotAPolicyContainer;
use crate::policy::{AnyPolicyTrait, NoPolicy, Policy, PolicyDyn};

pub trait AsNoReflection<'a> {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a) where 'a: 'r;
}
pub trait AsLeaf {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait + 'static);
}

// B: &'r mut dyn AnyPolicyTrait.
// B: Box<dyn AnyPolicyTrait>
pub enum PolicyReflection<'a, L: AsLeaf, NR: AsNoReflection<'a>> {
    NoReflection(NR),  // Something we cannot use reflection on, e.g. a type that is not std::any::Any.
    Leaf(L),
    PolicyAnd(Box<Self>, Box<Self>),
    PolicyOr(Box<Self>, Box<Self>),
    OptionPolicy(Option<Box<Self>>),
    AnyPolicy(Box<Self>),
    TestPolicy(Box<Self>),
    _Unreachable(Infallible, PhantomData<&'a ()>),
}

// This is a policy container.
impl<'r, 'a: 'r, L: AsLeaf, NR: AsNoReflection<'a>> !NotAPolicyContainer for PolicyReflection<'a, L, NR> {}

// Normalize a reflection enum (by removing AnyPolicies and TestPolicies)
impl<'r, 'a: 'r, L: AsLeaf, NR: AsNoReflection<'a>> PolicyReflection<'a, L, NR> {
    // Check if this is in effect a no policy.
    pub fn is_no_policy(&'r self) -> bool {
        match self {
            PolicyReflection::NoReflection(pol) => {
                let pol = pol.as_ref();
                false
            },
            PolicyReflection::Leaf(b) => {
                b.as_ref().upcast_ref().is::<NoPolicy>()
            }
            PolicyReflection::PolicyAnd(p1, p2) => {
                p1.is_no_policy() && p2.is_no_policy()
            },
            PolicyReflection::PolicyOr(p1, p2) => {
                p1.is_no_policy() || p2.is_no_policy()
            },
            PolicyReflection::OptionPolicy(p) => {
                match p {
                    None => true,
                    Some(p) => p.is_no_policy(),
                }
            },
            PolicyReflection::AnyPolicy(_) => panic!("use normalize() first"),
            PolicyReflection::TestPolicy(_) => panic!("use normalize() first"),
            PolicyReflection::_Unreachable(inf, _) => match *inf {},
        }
    }

    // Normalize representation by removing AnyPolicies.
    pub fn normalize(self) -> Self {
        match self {
            PolicyReflection::NoReflection(p) => PolicyReflection::NoReflection(p),
            PolicyReflection::Leaf(p) => PolicyReflection::Leaf(p),
            PolicyReflection::PolicyAnd(p1, p2) =>
                PolicyReflection::PolicyAnd(
                    Box::new(p1.normalize()),
                    Box::new(p2.normalize()),
                ),
            PolicyReflection::PolicyOr(p1, p2) =>
                PolicyReflection::PolicyOr(
                    Box::new(p1.normalize()),
                    Box::new(p2.normalize()),
                ),
            PolicyReflection::OptionPolicy(p) => match p {
                None => PolicyReflection::OptionPolicy(None),
                Some(p) => PolicyReflection::OptionPolicy(
                    Some(Box::new(p.normalize()))
                ),
            },
            PolicyReflection::AnyPolicy(p) => p.normalize(),
            PolicyReflection::TestPolicy(p) => p.normalize(),
            PolicyReflection::_Unreachable(inf, _) => match inf {},
        }
    }

    // Turn this reflection enum into reflection enum with reference leafs.
    pub fn deref_mut<'rr, L2: ?Sized, NR2: ?Sized>(&'rr mut self) -> PolicyReflection<'a, &'rr mut L2, &'rr mut NR2>
    where
        L: DerefMut<Target = L2>,
        NR: DerefMut<Target = NR2>,
        &'rr mut L2: AsLeaf,
        &'rr mut NR2: AsNoReflection<'a>,
    {
        match self {
            PolicyReflection::NoReflection(b) => {
                let b = (*b).deref_mut();
                PolicyReflection::NoReflection(b)
            },
            PolicyReflection::Leaf(b) => {
                let b = (*b).deref_mut();
                PolicyReflection::Leaf(b)
            },
            PolicyReflection::PolicyAnd(a, b) => {
                PolicyReflection::PolicyAnd(
                    Box::new(a.deref_mut().deref_mut()),
                    Box::new(b.deref_mut().deref_mut())
                )
            },
            PolicyReflection::PolicyOr(a, b) => {
                PolicyReflection::PolicyOr(
                    Box::new(a.deref_mut().deref_mut()),
                    Box::new(b.deref_mut().deref_mut())
                )
            },
            PolicyReflection::OptionPolicy(p) => match p {
                None => PolicyReflection::OptionPolicy(None),
                Some(p) => {
                    PolicyReflection::OptionPolicy(
                        Some(Box::new(p.deref_mut().deref_mut()))
                    )
                },
            },
            PolicyReflection::AnyPolicy(p) => {
                PolicyReflection::AnyPolicy(Box::new(p.deref_mut().deref_mut()))
            },
            PolicyReflection::TestPolicy(p) => {
                PolicyReflection::TestPolicy(Box::new(p.deref_mut().deref_mut()))
            },
            PolicyReflection::_Unreachable(inf, _) => match *inf {

            },
        }
    }
}