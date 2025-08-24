use crate::policy::{
    AsLeaf, AsNoReflection, ByMove, ByMutRef, ByRef, PassType, PolicyReflection, PostfixOutcome,
    PrefixOutcome, Visitor,
};
use std::marker::PhantomData;

// Prefix Helpers.
pub trait PrefixVisitor<'a, T: PassType<'a>> {
    type Result: Sized;

    // Reflection and Leaf have no children, prefix and postfix are the same.
    fn visit_no_reflection(&mut self, b: T::NoReflection, parent: Self::Result);
    fn visit_leaf(&mut self, b: T::Leaf, parent: Self::Result);

    // PolicyAnd.
    fn visit_and(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        parent: Self::Result,
    ) -> PrefixOutcome<T::Enum, Self::Result, ()>;

    // PolicyOr.
    fn visit_or(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        parent: Self::Result,
    ) -> PrefixOutcome<T::Enum, Self::Result, ()>;

    // PolicyRef.
    fn visit_ref(&mut self, p: T::NoReflection, e: T::NestedEnum, parent: Self::Result);

    // OptionPolicy.
    fn visit_option(
        &mut self,
        option: Option<T::Enum>,
        parent: Self::Result,
    ) -> PrefixOutcome<T::Enum, Self::Result, ()>;

    // AnyPolicyDyn.
    fn visit_any(
        &mut self,
        _policy: T::Enum,
        _parent: Self::Result,
    ) -> PrefixOutcome<T::Enum, Self::Result, ()> {
        panic!("Normalized reflection assumed; found AnyPolicyDyn<_>");
    }

    // TestPolicy.
    fn visit_test(
        &mut self,
        _policy: T::Enum,
        _parent: Self::Result,
    ) -> PrefixOutcome<T::Enum, Self::Result, ()> {
        panic!("Normalized reflection assumed; found TestPolicy");
    }
}

// Visitor driver.
impl<'a, L: AsLeaf, NR: AsNoReflection<'a>> PolicyReflection<'a, L, NR> {
    pub fn prefix_visit_by_move<V: PrefixVisitor<'a, ByMove<'a, L, NR>>>(
        self,
        v: &mut V,
        parent: V::Result,
    ) {
        let mut v = PrefixVisitorHelper::from(v);
        self.visit_by_move(&mut v, parent)
    }
    pub fn prefix_visit_by_ref<'r, V: PrefixVisitor<'a, ByRef<'r, 'a, L, NR>>>(
        &'r mut self,
        v: &mut V,
        parent: V::Result,
    ) where
        'a: 'r,
    {
        let mut v = PrefixVisitorHelper::from(v);
        self.visit_by_ref(&mut v, parent)
    }
    pub fn prefix_visit_by_mut_ref<'r, V: PrefixVisitor<'a, ByMutRef<'r, 'a, L, NR>>>(
        &'a mut self,
        v: &mut V,
        parent: V::Result,
    ) where
        'a: 'r,
    {
        let mut v = PrefixVisitorHelper::from(v);
        self.visit_by_mut_ref(&mut v, parent)
    }
}

// Prefix visitors are Visitors.
struct PrefixVisitorHelper<'r, 'a: 'r, 'b, T: PassType<'a>, V: PrefixVisitor<'a, T> + ?Sized> {
    visitor: &'b mut V,
    _data: PhantomData<&'a ()>,
    _data2: PhantomData<&'r ()>,
    _data3: PhantomData<T>,
}

impl<'r, 'a: 'r, 'b, T: PassType<'a>, V: PrefixVisitor<'a, T> + ?Sized> From<&'b mut V>
    for PrefixVisitorHelper<'r, 'a, 'b, T, V>
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
impl<'r, 'a: 'r, 'b, T: PassType<'a>, V: PrefixVisitor<'a, T> + ?Sized> Visitor<'a, T>
    for PrefixVisitorHelper<'r, 'a, 'b, T, V>
{
    type PrefixResult = V::Result;
    type PostfixResult = ();

    // Reflection and Leaf.
    fn visit_no_reflection(
        &mut self,
        b: T::NoReflection,
        parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_no_reflection(b, parent);
        Ok(())
    }
    fn visit_leaf(
        &mut self,
        b: T::Leaf,
        parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_leaf(b, parent);
        Ok(())
    }

    // PolicyAnd.
    fn visit_and_prefix(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        self.visitor.visit_and(left, right, parent)
    }
    fn visit_and_postfix(
        &mut self,
        _left: Self::PostfixResult,
        _right: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        Ok(())
    }

    // PolicyOr.
    fn visit_or_prefix(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        self.visitor.visit_or(left, right, parent)
    }
    fn visit_or_postfix(
        &mut self,
        _left: Self::PostfixResult,
        _right: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        Ok(())
    }

    // PolicyRef.
    fn visit_ref(
        &mut self,
        p: T::NoReflection,
        e: T::NestedEnum,
        parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        self.visitor.visit_ref(p, e, parent);
        Ok(())
    }

    // OptionPolicy.
    fn visit_option_prefix(
        &mut self,
        option: Option<T::Enum>,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        self.visitor.visit_option(option, parent)
    }
    fn visit_option_postfix(
        &mut self,
        _result: Option<Self::PostfixResult>,
    ) -> PostfixOutcome<Self::PostfixResult> {
        Ok(())
    }

    // AnyPolicyDyn.
    fn visit_any_prefix(
        &mut self,
        policy: T::Enum,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        self.visitor.visit_any(policy, parent)
    }
    fn visit_any_postfix(
        &mut self,
        _policy: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        Ok(())
    }

    // TestPolicy.
    fn visit_test_prefix(
        &mut self,
        policy: T::Enum,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        self.visitor.visit_test(policy, parent)
    }
    fn visit_test_postfix(
        &mut self,
        _policy: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        Ok(())
    }
}
