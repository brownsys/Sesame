use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::SandboxAllocator, ptr::*, swizzle::*, vec::{MyVec, NonNull, RawMyVec}, Sandboxable};
use chrono::naive::NaiveDateTime;


impl<T: Sandboxable> Sandboxable for Box<T> {
    type InSandboxUnswizzled = BoxUnswizzled<T::InSandboxUnswizzled, SandboxAllocator>; // TODO: is this the right memory layout

    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled {
        // 1. move boxed value into the sandbox portion of memory
        let new_val = Sandboxable::into_sandbox(*outside, alloc.clone());
        let b = Box::new_in(new_val, alloc);

        // 2. convert to a sandbox box w 32 bit ptr
        let ptr = Box::into_raw(b);
        let new_b = BoxUnswizzled { ptr: unswizzle_ptr(ptr), phantom_data: PhantomData::<SandboxAllocator> };
        new_b
    }

    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, any_sandbox_ptr: usize) -> Self {
        todo!();
    }
}

// Derive `Sandboxable` for primitives that won't change in the sandbox.
macro_rules! derive_sandboxable_identity {
    ($t:ty) => {
        impl Sandboxable for $t {
            type InSandboxUnswizzled = $t;
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

impl Sandboxable for usize {
    type InSandboxUnswizzled = u32;
    fn into_sandbox(outside: Self, _: SandboxAllocator) -> Self::InSandboxUnswizzled {
        outside.try_into().unwrap()
    }
    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, _: usize) -> Self {
        inside.clone().try_into().unwrap()
    }
}