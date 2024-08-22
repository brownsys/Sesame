use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::SandboxAllocator, ptr::*, swizzle::*, vec::{MyVec, NonNull, RawMyVec}, FastSandboxTransfer};
use chrono::naive::NaiveDateTime;

// Implement `FastSandboxTransfer` for primitives that won't change in the sandbox.
macro_rules! derive_sandboxable_identity {
    ($t:ty) => {
        #[doc = "Library implementation of FastSandboxTransfer. Do not copy this docstring!"]
        impl FastSandboxTransfer for $t {
            type InSandboxUnswizzled = $t;
            fn is_identity() -> bool { true }
            fn into_sandbox(outside: Self, _: SandboxAllocator) -> Self::InSandboxUnswizzled { outside }
            fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, _: usize) -> Self where Self: Sized { inside.clone() }
        }
    }
}

derive_sandboxable_identity!(());
derive_sandboxable_identity!(bool);
derive_sandboxable_identity!(u8);
derive_sandboxable_identity!(u16);
derive_sandboxable_identity!(u32);
derive_sandboxable_identity!(u64);
derive_sandboxable_identity!(i8);
derive_sandboxable_identity!(i16);
derive_sandboxable_identity!(i32);
derive_sandboxable_identity!(i64);
derive_sandboxable_identity!(f32);
derive_sandboxable_identity!(f64);
derive_sandboxable_identity!(NaiveDateTime);

#[doc = "Library implementation of FastSandboxTransfer. Do not copy this docstring!"]
impl FastSandboxTransfer for usize {
    type InSandboxUnswizzled = u32;
    fn into_sandbox(outside: Self, _: SandboxAllocator) -> u32 { outside.try_into().unwrap() }
    fn out_of_sandbox(inside: &u32, _: usize) -> Self { inside.clone().try_into().unwrap() }
}

#[doc = "Library implementation of FastSandboxTransfer. Do not copy this docstring!"]
impl FastSandboxTransfer for isize {
    type InSandboxUnswizzled = i32;
    fn into_sandbox(outside: Self, _: SandboxAllocator) -> i32 { outside.try_into().unwrap() }
    fn out_of_sandbox(inside: &i32, _: usize) -> Self { inside.clone().try_into().unwrap() }
}