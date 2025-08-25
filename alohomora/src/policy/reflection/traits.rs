use crate::policy::{
    AnyPolicy, AsLeaf, AsNoReflection, MutRefReflection, OptionPolicy, OwnedReflection, Policy,
    PolicyAnd, PolicyDyn, PolicyOr, PolicyReflection, RefPolicy, RefReflection, ToMutableRef,
    ToRef,
};
use crate::testing::TestPolicy;
use std::any::Any;


// Marks which types are not a policy container.
// Needed for Specialization, Joining, and RuntimeFoldIn to work.
pub auto trait NotAPolicyContainer {}

impl<'a, L: AsLeaf, NR: AsNoReflection<'a>> !NotAPolicyContainer for PolicyReflection<'a, L, NR> {}

impl<P: PolicyDyn + ?Sized> !NotAPolicyContainer for AnyPolicy<P> {}

impl<P: Policy> !NotAPolicyContainer for TestPolicy<P> {}

impl<'a, P: Policy + ?Sized> !NotAPolicyContainer for RefPolicy<'a, P> {}

impl<P1: Policy, P2: Policy> !NotAPolicyContainer for PolicyAnd<P1, P2> {}

impl<P1: Policy, P2: Policy> !NotAPolicyContainer for PolicyOr<P1, P2> {}

impl<P: Policy + Clone + 'static> !NotAPolicyContainer for OptionPolicy<P> {}

// Implementors of Reflective can be transformed into a PolicyReflection enum
// to allow reflection at runtime on the policy types/structure.
pub trait Reflective {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_>;
    fn reflect_ref(&self) -> RefReflection<'_>;
    // with static lifetime if Self lives long enough.
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static>
    where
        Self: 'static;
}
pub trait ReflectiveOwned<'a> {
    fn reflect_owned(self) -> OwnedReflection<'a>
    where
        Self: Sized;
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a>;
}

// It is already reflected!
impl<'a> Reflective for OwnedReflection<'a> {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        let mut v = ToMutableRef {};
        self.postfix_visit_by_mut_ref(&mut v)
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        let mut v = ToRef {};
        self.postfix_visit_by_ref(&mut v)
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static>
    where
        Self: 'static,
    {
        *self
    }
}
impl<'a> ReflectiveOwned<'a> for OwnedReflection<'a> {
    fn reflect_owned(self) -> OwnedReflection<'a> {
        self
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        *self
    }
}

// Leafs
impl<'a, P: Policy + Any + NotAPolicyContainer> ReflectiveOwned<'a> for P {
    fn reflect_owned(self) -> OwnedReflection<'a>
    where
        Self: Sized,
    {
        OwnedReflection::Leaf(Box::new(self))
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        OwnedReflection::Leaf(self)
    }
}
impl<P: Policy + Any + NotAPolicyContainer> Reflective for P {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        MutRefReflection::Leaf(self)
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        RefReflection::Leaf(self)
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static> {
        self.reflect_box()
    }
}

// PolicyAnd.
impl<'a, P1: Policy + ReflectiveOwned<'a>, P2: Policy + ReflectiveOwned<'a>> ReflectiveOwned<'a>
    for PolicyAnd<P1, P2>
{
    fn reflect_owned(self) -> OwnedReflection<'a> {
        let (p1, p2) = self.into_inner();
        OwnedReflection::PolicyAnd(Box::new(p1.reflect_owned()), Box::new(p2.reflect_owned()))
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        self.reflect_owned()
    }
}
impl<'a, P1: Policy, P2: Policy> Reflective for PolicyAnd<P1, P2> {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        let (p1, p2) = self.mut_policies();
        MutRefReflection::PolicyAnd(
            Box::new(p1.reflect_mut_ref()),
            Box::new(p2.reflect_mut_ref()),
        )
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        let (p1, p2) = self.policies();
        RefReflection::PolicyAnd(Box::new(p1.reflect_ref()), Box::new(p2.reflect_ref()))
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static>
    where
        Self: 'static,
    {
        let (p1, p2) = self.into_inner();
        let p1 = Box::new(p1);
        let p2 = Box::new(p2);
        OwnedReflection::PolicyAnd(Box::new(p1.reflect_static()), Box::new(p2.reflect_static()))
    }
}

// PolicyOr.
impl<'a, P1: Policy + ReflectiveOwned<'a>, P2: Policy + ReflectiveOwned<'a>> ReflectiveOwned<'a>
    for PolicyOr<P1, P2>
{
    fn reflect_owned(self) -> OwnedReflection<'a> {
        let (p1, p2) = self.into_inner();
        OwnedReflection::PolicyOr(Box::new(p1.reflect_owned()), Box::new(p2.reflect_owned()))
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        self.reflect_owned()
    }
}
impl<'a, P1: Policy, P2: Policy> Reflective for PolicyOr<P1, P2> {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        let (p1, p2) = self.mut_policies();
        MutRefReflection::PolicyOr(
            Box::new(p1.reflect_mut_ref()),
            Box::new(p2.reflect_mut_ref()),
        )
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        let (p1, p2) = self.policies();
        RefReflection::PolicyOr(Box::new(p1.reflect_ref()), Box::new(p2.reflect_ref()))
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static>
    where
        Self: 'static,
    {
        let (p1, p2) = self.into_inner();
        let p1 = Box::new(p1);
        let p2 = Box::new(p2);
        OwnedReflection::PolicyOr(Box::new(p1.reflect_static()), Box::new(p2.reflect_static()))
    }
}

// Can reflect on RefPolicy (but it is kind of a useless reflection/does not allow
// specialization unless the lifetime 'static!).
impl<'a, P: Policy + ?Sized> ReflectiveOwned<'a> for RefPolicy<'a, P> {
    fn reflect_owned(self) -> OwnedReflection<'a> {
        let pol = Box::new(self);
        let e = Box::new(pol.policy().reflect_ref());
        OwnedReflection::PolicyRef(pol, e)
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        let e = Box::new(self.policy().reflect_ref());
        OwnedReflection::PolicyRef(self, e)
    }
}
impl<'a, P: Policy + ?Sized> Reflective for RefPolicy<'a, P> {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        let e = Box::new(self.policy().reflect_ref());
        MutRefReflection::PolicyRef(self, e)
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        RefReflection::PolicyRef(self, Box::new(self.policy().reflect_ref()))
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static>
    where
        Self: 'static,
    {
        self.reflect_box()
    }
}
// OptionPolicy
impl<'a, P: Policy + ReflectiveOwned<'a>> ReflectiveOwned<'a> for OptionPolicy<P> {
    fn reflect_owned(self) -> OwnedReflection<'a> {
        match self {
            Self::NoPolicy => OwnedReflection::OptionPolicy(None),
            Self::Policy(p) => OwnedReflection::OptionPolicy(Some(Box::new(p.reflect_owned()))),
        }
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        self.reflect_owned()
    }
}
impl<'a, P: Policy> Reflective for OptionPolicy<P> {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        match self {
            Self::NoPolicy => MutRefReflection::OptionPolicy(None),
            Self::Policy(p) => MutRefReflection::OptionPolicy(Some(Box::new(p.reflect_mut_ref()))),
        }
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        match self {
            Self::NoPolicy => RefReflection::OptionPolicy(None),
            Self::Policy(p) => RefReflection::OptionPolicy(Some(Box::new(p.reflect_ref()))),
        }
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static>
    where
        Self: 'static,
    {
        match *self {
            OptionPolicy::NoPolicy => OwnedReflection::OptionPolicy(None),
            OptionPolicy::Policy(p) => {
                OwnedReflection::OptionPolicy(Some(Box::new(Box::new(p).reflect_static())))
            }
        }
    }
}

// AnyPolicyDyn
impl<'a, PDyn: PolicyDyn + ?Sized> ReflectiveOwned<'a> for AnyPolicy<PDyn> {
    fn reflect_owned(self) -> OwnedReflection<'a> {
        let b = self.into_inner();
        OwnedReflection::AnyPolicy(Box::new(b.reflect_static()))
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        self.reflect_owned()
    }
}
impl<'a, PDyn: PolicyDyn + ?Sized> Reflective for AnyPolicy<PDyn> {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        MutRefReflection::AnyPolicy(Box::new(self.mut_inner().reflect_mut_ref()))
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        RefReflection::AnyPolicy(Box::new(self.inner().reflect_ref()))
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static> {
        OwnedReflection::AnyPolicy(Box::new(Box::new(self.into_inner()).reflect_static()))
    }
}
// TestPolicy
impl<'a, P: Policy + ReflectiveOwned<'a>> ReflectiveOwned<'a> for TestPolicy<P> {
    fn reflect_owned(self) -> OwnedReflection<'a> {
        OwnedReflection::TestPolicy(Box::new(self.into_inner().reflect_owned()))
    }
    fn reflect_box(self: Box<Self>) -> OwnedReflection<'a> {
        self.reflect_owned()
    }
}
impl<'a, P: Policy> Reflective for TestPolicy<P> {
    fn reflect_mut_ref(&mut self) -> MutRefReflection<'_> {
        MutRefReflection::TestPolicy(Box::new(self.mut_policy().reflect_mut_ref()))
    }
    fn reflect_ref(&self) -> RefReflection<'_> {
        RefReflection::TestPolicy(Box::new(self.policy().reflect_ref()))
    }
    fn reflect_static(self: Box<Self>) -> OwnedReflection<'static>
    where
        Self: 'static,
    {
        OwnedReflection::TestPolicy(Box::new(Box::new(self.into_inner()).reflect_static()))
    }
}
