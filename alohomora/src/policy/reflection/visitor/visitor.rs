use crate::policy::reflection::visitor::visitor::driver::visit_helper;
use crate::policy::{AsLeaf, AsNoReflection, ByMove, ByMutRef, ByRef, PassType, PolicyReflection};




pub type PrefixOutcome<Enum, PrefixResult, PostfixResult> = Result<
    // Continue visiting this node's children. Because data is moved into the visitor,
    // the visitor function must return (1) the children, along with
    // (2) it's own intermediate output (each gets passed to the corresponding child).
    Vec<(Enum, PrefixResult)>,
    // Stop visiting this node (and children) and return this to parents.
    // Continues visiting parents in postfix order.
    PostfixResult,
>;
pub type PostfixOutcome<PostfixResult> = Result<
    // Take the result and pass it to parents postfix (continue visiting).
    PostfixResult,
    // Stop visiting parents, return this directly to caller.
    PostfixResult,
>;

pub trait Visitor<'a, T: PassType<'a>> {
    type PrefixResult: Sized;
    type PostfixResult: Sized;

    // Reflection and Leaf have no children, prefix and postfix are the same.
    fn visit_no_reflection(
        &mut self,
        b: T::NoReflection,
        parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult>;
    fn visit_leaf(
        &mut self,
        b: T::Leaf,
        parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult>;

    // PolicyAnd.
    fn visit_and_prefix(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult>;
    fn visit_and_postfix(
        &mut self,
        left: Self::PostfixResult,
        right: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult>;

    // PolicyOr.
    fn visit_or_prefix(
        &mut self,
        left: T::Enum,
        right: T::Enum,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult>;
    fn visit_or_postfix(
        &mut self,
        left: Self::PostfixResult,
        right: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult>;

    // RefPolicy.
    fn visit_ref(
        &mut self,
        p: T::NoReflection,
        e: T::NestedEnum,
        parent: Self::PrefixResult,
    ) -> PostfixOutcome<Self::PostfixResult>;

    // OptionPolicy.
    fn visit_option_prefix(
        &mut self,
        option: Option<T::Enum>,
        parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult>;
    fn visit_option_postfix(
        &mut self,
        result: Option<Self::PostfixResult>,
    ) -> PostfixOutcome<Self::PostfixResult>;

    // AnyPolicyDyn.
    fn visit_any_prefix(
        &mut self,
        _policy: T::Enum,
        _parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        panic!("Normalized reflection assumed; found AnyPolicyDyn<_>");
    }
    fn visit_any_postfix(
        &mut self,
        _policy: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        panic!("Normalized reflection assumed; found AnyPolicyDyn<_>");
    }

    // TestPolicy.
    fn visit_test_prefix(
        &mut self,
        _policy: T::Enum,
        _parent: Self::PrefixResult,
    ) -> PrefixOutcome<T::Enum, Self::PrefixResult, Self::PostfixResult> {
        panic!("Normalized reflection assumed; found TestPolicy");
    }
    fn visit_test_postfix(
        &mut self,
        _policy: Self::PostfixResult,
    ) -> PostfixOutcome<Self::PostfixResult> {
        panic!("Normalized reflection assumed; found TestPolicy");
    }
}

// Visitor driver.
impl<'a, L: AsLeaf, NR: AsNoReflection<'a>> PolicyReflection<'a, L, NR> {
    pub fn visit_by_move<V: Visitor<'a, ByMove<'a, L, NR>>>(
        self,
        v: &mut V,
        parent: V::PrefixResult,
    ) -> V::PostfixResult {
        visit_helper(self, v, parent).unwrap_or_else(|err| err)
    }

    pub fn visit_by_ref<'r, V: Visitor<'a, ByRef<'r, 'a, L, NR>>>(
        &'r self,
        v: &mut V,
        parent: V::PrefixResult,
    ) -> V::PostfixResult
    where
        'a: 'r,
    {
        visit_helper(self, v, parent).unwrap_or_else(|err| err)
    }

    pub fn visit_by_mut_ref<'r, V: Visitor<'a, ByMutRef<'r, 'a, L, NR>>>(
        &'r mut self,
        v: &mut V,
        parent: V::PrefixResult,
    ) -> V::PostfixResult
    where
        'a: 'r,
    {
        visit_helper(self, v, parent).unwrap_or_else(|err| err)
    }
}

mod driver {
    use crate::policy::{
        AsLeaf, AsNoReflection, ByMove, ByMutRef, ByRef, PassType,
        PolicyReflection, PostfixOutcome, Visitor,
    };
    use itertools::Itertools;
    use std::convert::Infallible;
    use std::marker::PhantomData;

    // Allow us to get an Enum of the same type when we match.
    enum OneLevelEnum<'r, T: PassType<'r>> {
        NoReflection(T::NoReflection), // Something we cannot use reflection on, e.g. a type that is not std::any::Any.
        Leaf(T::Leaf),
        PolicyAnd(T::Enum, T::Enum),
        PolicyOr(T::Enum, T::Enum),
        PolicyRef(T::NoReflection, T::NestedEnum),
        OptionPolicy(Option<T::Enum>),
        AnyPolicy(T::Enum),
        TestPolicy(T::Enum),
        _Unreachable(Infallible, PhantomData<&'r ()>),
    }

    // One level!
    impl<'a, L, NR> From<<ByMove<'a, L, NR> as PassType<'a>>::Enum>
        for OneLevelEnum<'a, ByMove<'a, L, NR>>
    where
        L: AsLeaf,
        NR: AsNoReflection<'a>,
    {
        fn from(value: <ByMove<'a, L, NR> as PassType<'a>>::Enum) -> Self {
            match value {
                PolicyReflection::NoReflection(pol) => OneLevelEnum::NoReflection(pol),
                PolicyReflection::Leaf(b) => OneLevelEnum::Leaf(b),
                PolicyReflection::PolicyAnd(p1, p2) => OneLevelEnum::PolicyAnd(*p1, *p2),
                PolicyReflection::PolicyOr(p1, p2) => OneLevelEnum::PolicyOr(*p1, *p2),
                PolicyReflection::PolicyRef(p, e) => OneLevelEnum::PolicyRef(p, *e),
                PolicyReflection::OptionPolicy(p) => OneLevelEnum::OptionPolicy(p.map(|b| *b)),
                PolicyReflection::AnyPolicy(p) => OneLevelEnum::AnyPolicy(*p),
                PolicyReflection::TestPolicy(p) => OneLevelEnum::TestPolicy(*p),
                PolicyReflection::_Unreachable(p, _) => match p {},
            }
        }
    }
    impl<'r, 'a: 'r, L, NR> From<<ByMutRef<'r, 'a, L, NR> as PassType<'a>>::Enum>
        for OneLevelEnum<'a, ByMutRef<'r, 'a, L, NR>>
    where
        L: AsLeaf,
        NR: AsNoReflection<'a>,
    {
        fn from(value: <ByMutRef<'r, 'a, L, NR> as PassType<'a>>::Enum) -> Self {
            match value {
                PolicyReflection::NoReflection(pol) => OneLevelEnum::NoReflection(pol),
                PolicyReflection::Leaf(b) => OneLevelEnum::Leaf(b),
                PolicyReflection::PolicyAnd(p1, p2) => {
                    OneLevelEnum::PolicyAnd(p1.as_mut(), p2.as_mut())
                }
                PolicyReflection::PolicyOr(p1, p2) => {
                    OneLevelEnum::PolicyOr(p1.as_mut(), p2.as_mut())
                }
                PolicyReflection::PolicyRef(p, e) => OneLevelEnum::PolicyRef(p, e.as_mut()),
                PolicyReflection::OptionPolicy(p) => {
                    OneLevelEnum::OptionPolicy(p.as_mut().map(Box::as_mut))
                }
                PolicyReflection::AnyPolicy(p) => OneLevelEnum::AnyPolicy(p.as_mut()),
                PolicyReflection::TestPolicy(p) => OneLevelEnum::TestPolicy(p.as_mut()),
                PolicyReflection::_Unreachable(p, _) => match *p {},
            }
        }
    }

    impl<'r, 'a: 'r, L, NR> From<<ByRef<'r, 'a, L, NR> as PassType<'a>>::Enum>
        for OneLevelEnum<'a, ByRef<'r, 'a, L, NR>>
    where
        L: AsLeaf,
        NR: AsNoReflection<'a>,
    {
        fn from(value: <ByRef<'r, 'a, L, NR> as PassType<'a>>::Enum) -> Self {
            match value {
                PolicyReflection::NoReflection(pol) => OneLevelEnum::NoReflection(pol),
                PolicyReflection::Leaf(b) => OneLevelEnum::Leaf(b),
                PolicyReflection::PolicyAnd(p1, p2) => {
                    OneLevelEnum::PolicyAnd(p1.as_ref(), p2.as_ref())
                }
                PolicyReflection::PolicyOr(p1, p2) => {
                    OneLevelEnum::PolicyOr(p1.as_ref(), p2.as_ref())
                }
                PolicyReflection::PolicyRef(p, e) => OneLevelEnum::PolicyRef(p, e.as_ref()),
                PolicyReflection::OptionPolicy(p) => {
                    OneLevelEnum::OptionPolicy(p.as_ref().map(|b| b.as_ref()))
                }
                PolicyReflection::AnyPolicy(p) => OneLevelEnum::AnyPolicy(p.as_ref()),
                PolicyReflection::TestPolicy(p) => OneLevelEnum::TestPolicy(p.as_ref()),
                PolicyReflection::_Unreachable(p, _) => match *p {},
            }
        }
    }

    #[allow(private_bounds)]
    pub fn visit_helper<'a, T: PassType<'a>, V: Visitor<'a, T>>(
        e: T::Enum,
        visitor: &mut V,
        parent: V::PrefixResult,
    ) -> PostfixOutcome<V::PostfixResult>
    where
        OneLevelEnum<'a, T>: From<T::Enum>,
    {
        match OneLevelEnum::from(e) {
            // Leafs
            OneLevelEnum::NoReflection(b) => visitor.visit_no_reflection(b, parent),
            OneLevelEnum::Leaf(b) => visitor.visit_leaf(b, parent),
            // PolicyAnd.
            OneLevelEnum::PolicyAnd(p1, p2) => {
                let v = visitor.visit_and_prefix(p1, p2, parent)?;
                let v = visit_children(visitor, v)?;
                let (o1, o2) = v.into_iter().collect_tuple().unwrap();
                visitor.visit_and_postfix(o1, o2)
            }
            // PolicyOr.
            OneLevelEnum::PolicyOr(p1, p2) => {
                let v = visitor.visit_or_prefix(p1, p2, parent)?;
                let v = visit_children(visitor, v)?;
                let (o1, o2) = v.into_iter().collect_tuple().unwrap();
                visitor.visit_or_postfix(o1, o2)
            }
            OneLevelEnum::PolicyRef(p, e) => visitor.visit_ref(p, e, parent),
            OneLevelEnum::OptionPolicy(option) => match option {
                None => {
                    visitor.visit_option_prefix(None, parent)?;
                    visitor.visit_option_postfix(None)
                }
                Some(p) => {
                    let v = visitor.visit_option_prefix(Some(p), parent)?;
                    let v = visit_children(visitor, v)?;
                    let (o,) = v.into_iter().collect_tuple().unwrap();
                    visitor.visit_option_postfix(Some(o))
                }
            },
            OneLevelEnum::AnyPolicy(policy) => {
                let v = visitor.visit_any_prefix(policy, parent)?;
                let v = visit_children(visitor, v)?;
                let (o,) = v.into_iter().collect_tuple().unwrap();
                visitor.visit_any_postfix(o)
            }
            OneLevelEnum::TestPolicy(policy) => {
                let v = visitor.visit_test_prefix(policy, parent)?;
                let v = visit_children(visitor, v)?;
                let (o,) = v.into_iter().collect_tuple().unwrap();
                visitor.visit_test_postfix(o)
            }
            OneLevelEnum::_Unreachable(p, _) => match p {},
        }
    }

    // Helper function: visits children then the node in post-fix order.
    fn visit_children<'a, T: PassType<'a>, V: Visitor<'a, T>>(
        visitor: &mut V,
        children: Vec<(T::Enum, V::PrefixResult)>,
    ) -> Result<Vec<V::PostfixResult>, V::PostfixResult>
    where
        OneLevelEnum<'a, T>: From<T::Enum>,
    {
        let mut results = Vec::with_capacity(children.len());
        for (child, parent) in children {
            let result = visit_helper(child, visitor, parent)?;
            results.push(result);
        }
        Ok(results)
    }
}