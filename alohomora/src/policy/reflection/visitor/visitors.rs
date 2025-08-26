use crate::context::UnprotectedContext;
use crate::policy::{
    AnyPolicyDyn, AsLeaf, AsNoReflection, ByMove, ByMutRef, ByRef, MutRefReflection, NoPolicy,
    Policy, PolicyReflection, PostfixOutcome, PostfixVisitor, Reason, RefReflection,
};
use std::ops::Deref;

pub struct NormalizeVisitor {}
impl<'a, L: AsLeaf, NR: AsNoReflection<'a>> PostfixVisitor<'a, ByMove<'a, L, NR>>
    for NormalizeVisitor
{
    type Result = PolicyReflection<'a, L, NR>;

    fn visit_no_reflection(&mut self, b: NR) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::NoReflection(b))
    }
    fn visit_leaf(&mut self, b: L) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::Leaf(b))
    }
    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::PolicyAnd(Box::new(left), Box::new(right)))
    }
    fn visit_or(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::PolicyOr(Box::new(left), Box::new(right)))
    }
    fn visit_ref(&mut self, p: NR, e: RefReflection<'a>) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::PolicyRef(p, Box::new(e)))
    }
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::OptionPolicy(option.map(Box::new)))
    }
    fn visit_any(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(policy)
    }
    fn visit_test(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(policy)
    }
}

pub struct NameVisitor {}
impl<'r, 'a: 'r, L: AsLeaf + 'a, NR: AsNoReflection<'a> + 'a>
    PostfixVisitor<'a, ByRef<'r, 'a, L, NR>> for NameVisitor
{
    type Result = String;

    fn visit_no_reflection(&mut self, pol: &NR) -> PostfixOutcome<Self::Result> {
        Ok(pol.as_ref().name())
    }
    fn visit_leaf(&mut self, b: &L) -> PostfixOutcome<Self::Result> {
        Ok(b.as_ref().name())
    }

    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(format!("PolicyAnd({} AND {})", left, right))
    }
    fn visit_or(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(format!("PolicyOr({} OR {})", left, right))
    }

    fn visit_ref(&mut self, p: &NR, _e: &RefReflection<'a>) -> PostfixOutcome<Self::Result> {
        Ok(p.as_ref().name())
    }
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result> {
        match option {
            None => Ok(String::from("OptionPolicy(Empty)")),
            Some(policy) => Ok(format!("OptionPolicy({})", policy)),
        }
    }

    fn visit_any(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(format!("AnyPolicy({})", policy))
    }
    fn visit_test(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(format!("TestPolicy({})", policy))
    }
}

pub struct CheckVisitor<'a> {
    context: &'a UnprotectedContext,
    reason: Reason<'a>,
}
impl<'a> CheckVisitor<'a> {
    pub fn new(context: &'a UnprotectedContext, reason: Reason<'a>) -> Self {
        Self { context, reason }
    }
}
impl<'r, 'a: 'r, 'c, L: AsLeaf + 'a, NR: AsNoReflection<'a> + 'a>
    PostfixVisitor<'a, ByRef<'r, 'a, L, NR>> for CheckVisitor<'c>
{
    type Result = bool;

    fn visit_no_reflection(&mut self, pol: &NR) -> PostfixOutcome<Self::Result> {
        Ok(pol.as_ref().check(self.context, self.reason.clone()))
    }
    fn visit_leaf(&mut self, b: &L) -> PostfixOutcome<Self::Result> {
        Ok(b.as_ref()
            .upcast_policy()
            .check(self.context, self.reason.clone()))
    }
    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(left && right)
    }
    fn visit_or(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(left || right)
    }
    fn visit_ref(&mut self, p: &NR, _e: &RefReflection) -> PostfixOutcome<Self::Result> {
        Ok(p.as_ref().check(self.context, self.reason.clone()))
    }
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result> {
        Ok(option.unwrap_or(true))
    }
    fn visit_any(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(policy)
    }
    fn visit_test(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(policy)
    }
}

pub struct IsNoPolicy {}
impl<'r, 'a: 'r, L: AsLeaf + 'a, NR: AsNoReflection<'a> + 'a>
    PostfixVisitor<'a, ByRef<'r, 'a, L, NR>> for IsNoPolicy
{
    type Result = bool;

    fn visit_no_reflection(&mut self, _pol: &NR) -> PostfixOutcome<Self::Result> {
        Ok(false)
    }
    fn visit_leaf(&mut self, b: &L) -> PostfixOutcome<Self::Result> {
        Ok(b.as_ref().upcast_any().is::<NoPolicy>())
    }
    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(left && right)
    }
    fn visit_or(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(left || right)
    }
    fn visit_ref(&mut self, _p: &NR, e: &RefReflection) -> PostfixOutcome<Self::Result> {
        Ok(e.postfix_visit_by_ref(self))
    }
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result> {
        Ok(option.unwrap_or(true))
    }
    fn visit_any(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(policy)
    }
    fn visit_test(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(policy)
    }
}

pub struct ToRef {}
impl<'r, 'a: 'r> PostfixVisitor<'a, ByRef<'r, 'a, Box<dyn AnyPolicyDyn>, Box<dyn Policy + 'a>>>
    for ToRef
{
    type Result = RefReflection<'r>;

    fn visit_no_reflection(&mut self, b: &'r Box<dyn Policy + 'a>) -> PostfixOutcome<Self::Result> {
        Ok(RefReflection::NoReflection(b.deref()))
    }
    fn visit_leaf(&mut self, b: &'r Box<dyn AnyPolicyDyn>) -> PostfixOutcome<Self::Result> {
        Ok(RefReflection::Leaf(b.deref()))
    }

    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(RefReflection::PolicyAnd(Box::new(left), Box::new(right)))
    }
    fn visit_or(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(RefReflection::PolicyOr(Box::new(left), Box::new(right)))
    }
    fn visit_ref(
        &mut self,
        p: &'r Box<dyn Policy + 'a>,
        e: &'r RefReflection<'a>,
    ) -> PostfixOutcome<Self::Result> {
        let mut v = CloneVisitor {};
        Ok(RefReflection::PolicyRef(
            p.deref(),
            Box::new(e.postfix_visit_by_ref(&mut v)),
        ))
    }
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result> {
        match option {
            None => Ok(RefReflection::OptionPolicy(None)),
            Some(p) => Ok(RefReflection::OptionPolicy(Some(Box::new(p)))),
        }
    }
    fn visit_any(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(RefReflection::AnyPolicy(Box::new(policy)))
    }
    fn visit_test(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(RefReflection::TestPolicy(Box::new(policy)))
    }
}

pub struct ToMutableRef {}
impl<'r, 'a: 'r> PostfixVisitor<'a, ByMutRef<'r, 'a, Box<dyn AnyPolicyDyn>, Box<dyn Policy + 'a>>>
    for ToMutableRef
{
    type Result = MutRefReflection<'r>;

    fn visit_no_reflection(
        &mut self,
        b: &'r mut Box<dyn Policy + 'a>,
    ) -> PostfixOutcome<Self::Result> {
        Ok(MutRefReflection::NoReflection(b.as_mut()))
    }
    fn visit_leaf(&mut self, b: &'r mut Box<dyn AnyPolicyDyn>) -> PostfixOutcome<Self::Result> {
        Ok(MutRefReflection::Leaf(b.as_mut()))
    }

    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(MutRefReflection::PolicyAnd(Box::new(left), Box::new(right)))
    }
    fn visit_or(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(MutRefReflection::PolicyOr(Box::new(left), Box::new(right)))
    }
    fn visit_ref(
        &mut self,
        p: &'r mut Box<dyn Policy + 'a>,
        e: &'r mut RefReflection<'a>,
    ) -> PostfixOutcome<Self::Result> {
        let mut v = CloneVisitor {};
        Ok(MutRefReflection::PolicyRef(
            p.as_mut(),
            Box::new(e.postfix_visit_by_ref(&mut v)),
        ))
    }
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result> {
        match option {
            None => Ok(MutRefReflection::OptionPolicy(None)),
            Some(p) => Ok(MutRefReflection::OptionPolicy(Some(Box::new(p)))),
        }
    }
    fn visit_any(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(MutRefReflection::AnyPolicy(Box::new(policy)))
    }
    fn visit_test(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(MutRefReflection::TestPolicy(Box::new(policy)))
    }
}

pub struct CloneVisitor {}
impl<'a, 'r: 'a> PostfixVisitor<'a, ByRef<'r, 'a, &'a (dyn AnyPolicyDyn), &'a (dyn Policy + 'a)>>
    for CloneVisitor
{
    type Result = PolicyReflection<'r, &'a (dyn AnyPolicyDyn), &'a (dyn Policy + 'a)>;

    fn visit_no_reflection(
        &mut self,
        b: &'r &'a (dyn Policy + 'a),
    ) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::NoReflection(*b))
    }
    fn visit_leaf(&mut self, b: &'r &'a (dyn AnyPolicyDyn)) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::Leaf(*b))
    }

    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::PolicyAnd(Box::new(left), Box::new(right)))
    }
    fn visit_or(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::PolicyOr(Box::new(left), Box::new(right)))
    }

    fn visit_ref(
        &mut self,
        p: &'r &'a (dyn Policy + 'a),
        e: &'r RefReflection<'a>,
    ) -> PostfixOutcome<Self::Result> {
        let e = e.postfix_visit_by_ref(self);
        Ok(PolicyReflection::PolicyRef(*p, Box::new(e)))
    }
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::OptionPolicy(option.map(Box::new)))
    }

    fn visit_any(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::AnyPolicy(Box::new(policy)))
    }
    fn visit_test(&mut self, policy: Self::Result) -> PostfixOutcome<Self::Result> {
        Ok(PolicyReflection::TestPolicy(Box::new(policy)))
    }
}
