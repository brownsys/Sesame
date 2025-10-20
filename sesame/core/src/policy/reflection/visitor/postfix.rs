use crate::policy::{
    AsLeaf, AsNoReflection, ByMove, ByMutRef, ByRef, PassType, PolicyReflection, PostfixOutcome,
    PrefixOutcome, Visitor,
};
use std::marker::PhantomData;

// Postfix Helpers.
pub trait PostfixVisitor<'a, T: PassType<'a>> {
    type Result: Sized;

    // Reflection and Leaf have no children, prefix and postfix are the same.
    fn visit_no_reflection(&mut self, b: T::NoReflection) -> PostfixOutcome<Self::Result>;
    fn visit_leaf(&mut self, b: T::Leaf) -> PostfixOutcome<Self::Result>;

    // PolicyAnd.
    fn visit_and(
        &mut self,
        left: Self::Result,
        right: Self::Result,
    ) -> PostfixOutcome<Self::Result>;

    // PolicyOr.
    fn visit_or(&mut self, left: Self::Result, right: Self::Result)
        -> PostfixOutcome<Self::Result>;

    // PolicyRef.
    fn visit_ref(&mut self, p: T::NoReflection, e: T::NestedEnum) -> PostfixOutcome<Self::Result>;

    // OptionPolicy.
    fn visit_option(&mut self, option: Option<Self::Result>) -> PostfixOutcome<Self::Result>;

    // AnyPolicyDyn.
    fn visit_any(&mut self, _policy: Self::Result) -> PostfixOutcome<Self::Result> {
        panic!("Normalized reflection assumed; found AnyPolicyDyn<_>");
    }

    // TestPolicy.
    fn visit_test(&mut self, _policy: Self::Result) -> PostfixOutcome<Self::Result> {
        panic!("Normalized reflection assumed; found TestPolicy");
    }
}

// Visitor driver.
impl<'a, L: AsLeaf, NR: AsNoReflection<'a>> PolicyReflection<'a, L, NR> {
    pub fn postfix_visit_by_move<V: PostfixVisitor<'a, ByMove<'a, L, NR>>>(
        self,
        v: &mut V,
    ) -> V::Result {
        let mut v = PostfixVisitorHelper::from(v);
        self.visit_by_move(&mut v, ())
    }
    pub fn postfix_visit_by_ref<'r, V: PostfixVisitor<'a, ByRef<'r, 'a, L, NR>>>(
        &'r self,
        v: &mut V,
    ) -> V::Result
    where
        'a: 'r,
    {
        let mut v = PostfixVisitorHelper::from(v);
        self.visit_by_ref(&mut v, ())
    }
    pub fn postfix_visit_by_mut_ref<'r, V: PostfixVisitor<'a, ByMutRef<'r, 'a, L, NR>>>(
        &'r mut self,
        v: &mut V,
    ) -> V::Result
    where
        'a: 'r,
    {
        let mut v = PostfixVisitorHelper::from(v);
        self.visit_by_mut_ref(&mut v, ())
    }
}

// Postfix Visitors are Visitors.
struct PostfixVisitorHelper<'r, 'a: 'r, 'b, T: PassType<'a>, V: PostfixVisitor<'a, T> + ?Sized> {
    visitor: &'b mut V,
    _data: PhantomData<&'a ()>,
    _data2: PhantomData<&'r ()>,
    _data3: PhantomData<T>,
}
impl<'r, 'a: 'r, 'b, T: PassType<'a>, V: PostfixVisitor<'a, T>>
    PostfixVisitorHelper<'r, 'a, 'b, T, V>
{
    fn from(visitor: &'b mut V) -> Self {
        Self {
            visitor,
            _data: PhantomData,
            _data2: PhantomData,
            _data3: PhantomData,
        }
    }
}
impl<'r, 'a: 'r, 'b, T: PassType<'a>, V: PostfixVisitor<'a, T> + ?Sized> Visitor<'a, T>
    for PostfixVisitorHelper<'r, 'a, 'b, T, V>
{
    type PrefixResult = ();
    type PostfixResult = V::Result;

    // Reflection and Leaf.
    fn visit_no_reflection(
        &mut self,
        b: T::NoReflection,
        _parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_no_reflection(b)
    }
    fn visit_leaf(
        &mut self,
        b: T::Leaf,
        _parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_leaf(b)
    }

    // PolicyAnd.
    fn visit_and_prefix(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        _parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        Ok(vec![(left, ()), (right, ())])
    }
    fn visit_and_postfix(
        &mut self,
        left: Self::PostfixResult,
        right: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_and(left, right)
    }

    // PolicyOr.
    fn visit_or_prefix(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        _parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        Ok(vec![(left, ()), (right, ())])
    }
    fn visit_or_postfix(
        &mut self,
        left: Self::PostfixResult,
        right: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_or(left, right)
    }

    // RefPolicy.
    fn visit_ref(
        &mut self,
        p: T::NoReflection,
        e: T::NestedEnum,
        _parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_ref(p, e)
    }

    // OptionPolicy.
    fn visit_option_prefix(
        &mut self,
        option: Option<T::Enum>,
        _parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        match option {
            None => Ok(vec![]),
            Some(p) => Ok(vec![(p, ())]),
        }
    }
    fn visit_option_postfix(
        &mut self,
        result: Option<Self::PostfixResult>,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_option(result)
    }

    // AnyPolicyDyn.
    fn visit_any_prefix(
        &mut self,
        policy: T::Enum,
        _parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        Ok(vec![(policy, ())])
    }
    fn visit_any_postfix(
        &mut self,
        policy: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_any(policy)
    }

    // TestPolicy.
    fn visit_test_prefix(
        &mut self,
        policy: T::Enum,
        _parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        Ok(vec![(policy, ())])
    }
    fn visit_test_postfix(
        &mut self,
        policy: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_test(policy)
    }
}
