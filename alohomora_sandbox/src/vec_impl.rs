use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::SandboxAllocator, ptr::*, swizzle::*, vec::{MyVec, NonNull, RawMyVec}};
use crate::Sandboxable;
use chrono::naive::NaiveDateTime;

impl<T: Sandboxable + Debug> Sandboxable for Vec<T> 
where T::InSandboxUnswizzled: Debug {
    type InSandboxUnswizzled = MyVecUnswizzled<T::InSandboxUnswizzled>;

    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled {
        // 1. move everything inside to sandbox
        let mut sandbox_vec = Vec::new_in(alloc.clone());
        outside.into_iter().map(|b|{
            Sandboxable::into_sandbox(b, alloc.clone())
        }).collect_into(&mut sandbox_vec);

        // 1b. convert to myvec so we can access private members
        let ptr: *const Vec<T::InSandboxUnswizzled, SandboxAllocator> = &sandbox_vec as *const Vec<T::InSandboxUnswizzled, SandboxAllocator>;
        let ptr = ptr as *mut MyVec<T::InSandboxUnswizzled, SandboxAllocator>;
        let a = unsafe { (*ptr).clone() };

        // 2. swizzle our metadata on the stack
        unswizzle_myvec(a)
    }

    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, sandbox_ptr: usize) -> Self where Self: Sized {
        // 2. swizzle our metadata on the stack
        let new_stack = swizzle_myvec(inside, sandbox_ptr);
        
        // 1b. convert back to real vec
        let p = Box::into_raw(Box::new(new_stack)) as *mut Vec<T::InSandboxUnswizzled>;
        let v = unsafe{ Box::leak(Box::from_raw(p)) };

        // 1. recursively bring all items out of the sandbox
        v.iter().map(|u| {
            Sandboxable::out_of_sandbox(u, sandbox_ptr)
        }).collect::<Vec<T>>()
    }
}
