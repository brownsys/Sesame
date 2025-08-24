use crate::policy::{AnyPolicyTrait, Policy};

// Whenever we are doing reflection, the leafs will meet these traits,
// allowing us immutable access to the underlying leaf policies.
pub trait AsNoReflection<'a> {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a)
    where
        'a: 'r;
}
pub trait AsLeaf {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait);
}

// Mutable version.
// This is not guaranteed to be the case when doing reflection.
// It must be explicitly added as a bound when reflection
// Requires mut ref access to leafs.
pub trait AsMutNoReflection<'a>: AsNoReflection<'a> + Sync + Send {
    fn as_mut_ref<'r>(&'r mut self) -> &'r mut (dyn Policy + 'a)
    where
        'a: 'r;
}
pub trait AsMutLeaf: AsLeaf + Sync + Send {
    fn as_mut_ref(&mut self) -> &mut (dyn AnyPolicyTrait);
}

// Boxes implement both.
impl AsLeaf for Box<dyn AnyPolicyTrait> {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait) {
        &**self
    }
}
impl<'a> AsNoReflection<'a> for Box<(dyn Policy + 'a)> {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a)
    where
        'a: 'r,
    {
        &**self
    }
}
impl AsMutLeaf for Box<dyn AnyPolicyTrait> {
    fn as_mut_ref(&mut self) -> &mut (dyn AnyPolicyTrait) {
        &mut **self
    }
}
impl<'a> AsMutNoReflection<'a> for Box<(dyn Policy + 'a)> {
    fn as_mut_ref<'r>(&'r mut self) -> &'r mut (dyn Policy + 'a)
    where
        'a: 'r,
    {
        &mut **self
    }
}

// Mutable references implement both.
impl AsMutLeaf for &mut (dyn AnyPolicyTrait) {
    fn as_mut_ref(&mut self) -> &mut (dyn AnyPolicyTrait) {
        &mut **self
    }
}
impl<'a> AsMutNoReflection<'a> for &mut (dyn Policy + 'a) {
    fn as_mut_ref<'r>(&'r mut self) -> &'r mut (dyn Policy + 'a)
    where
        'a: 'r,
    {
        &mut **self
    }
}
impl AsLeaf for &mut (dyn AnyPolicyTrait) {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait) {
        &**self
    }
}
impl<'a> AsNoReflection<'a> for &mut (dyn Policy + 'a) {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a)
    where
        'a: 'r,
    {
        &**self
    }
}

// Immutable references implement only immutable traits.
impl AsLeaf for &(dyn AnyPolicyTrait) {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait) {
        &**self
    }
}
impl<'a> AsNoReflection<'a> for &(dyn Policy + 'a) {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a)
    where
        'a: 'r,
    {
        &**self
    }
}

// Immutable refs of things that already implement the immutable traits also implement the immutable trait.
impl<T: AsLeaf + ?Sized> AsLeaf for &T {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait) {
        T::as_ref(*self)
    }
}
impl<'a, T: AsNoReflection<'a> + ?Sized> AsNoReflection<'a> for &T {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a)
    where
        'a: 'r,
    {
        T::as_ref(*self)
    }
}

// Mutable refs of things already implementing either traits implement that trait.
impl<T: AsLeaf + ?Sized> AsLeaf for &mut T {
    fn as_ref(&self) -> &(dyn AnyPolicyTrait) {
        T::as_ref(*self)
    }
}
impl<'a, T: AsNoReflection<'a> + ?Sized> AsNoReflection<'a> for &mut T {
    fn as_ref<'r>(&'r self) -> &'r (dyn Policy + 'a)
    where
        'a: 'r,
    {
        T::as_ref(*self)
    }
}
impl<T: AsMutLeaf + ?Sized> AsMutLeaf for &mut T {
    fn as_mut_ref(&mut self) -> &mut (dyn AnyPolicyTrait) {
        T::as_mut_ref(*self)
    }
}
impl<'a, T: AsMutNoReflection<'a> + ?Sized> AsMutNoReflection<'a> for &mut T {
    fn as_mut_ref<'r>(&'r mut self) -> &'r mut (dyn Policy + 'a)
    where
        'a: 'r,
    {
        T::as_mut_ref(*self)
    }
}
