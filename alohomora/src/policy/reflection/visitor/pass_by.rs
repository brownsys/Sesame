use std::marker::PhantomData;
use std::ops::Deref;
use crate::policy::{AsLeaf, AsNoReflection, PolicyReflection, RefReflection};

// Main visitor trait (does prefix + postfix).
pub trait PassType<'a> {
    type NoReflection: AsNoReflection<'a>;
    type Leaf: AsLeaf;
    type Enum;
    type NestedEnum;
}
pub struct ByMove<'a, L: AsLeaf, NR: AsNoReflection<'a>> {
    _data: PhantomData<PolicyReflection<'a, L, NR>>,
}
impl<'a, L: AsLeaf, NR: AsNoReflection<'a>> PassType<'a> for ByMove<'a, L, NR> {
    type NoReflection = NR;
    type Leaf = L;
    type Enum = PolicyReflection<'a, L, NR>;
    type NestedEnum = RefReflection<'a>;
}

pub struct ByMutRef<'r, 'a: 'r, L: AsLeaf, NR: AsNoReflection<'a>> {
    _data1: PhantomData<&'r ()>,
    _data2: PhantomData<&'a ()>,
    _data3: PhantomData<L>,
    _data4: PhantomData<NR>,
}
impl<'r, 'a: 'r, L: AsLeaf + 'r, NR: AsNoReflection<'a> + 'r> PassType<'a> for ByMutRef<'r, 'a, L, NR> {
    type NoReflection = &'r mut NR;
    type Leaf = &'r mut L;
    type Enum = &'r mut PolicyReflection<'a, L, NR>;
    type NestedEnum = &'r mut RefReflection<'a>;
}

pub struct ByRef<'r, 'a: 'r, L: AsLeaf, NR: AsNoReflection<'a>> {
    _data1: PhantomData<&'r ()>,
    _data2: PhantomData<&'a ()>,
    _data3: PhantomData<L>,
    _data4: PhantomData<NR>,
}
impl<'r, 'a: 'r, L: AsLeaf + 'r, NR: AsNoReflection<'a> + 'r> PassType<'a> for ByRef<'r, 'a, L, NR> {
    type NoReflection = &'r NR;
    type Leaf = &'r L;
    type Enum = &'r PolicyReflection<'a, L, NR>;
    type NestedEnum = &'r RefReflection<'a>;
}
