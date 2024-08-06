use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::SandboxAllocator, ptr::*, swizzle::*, vec::{MyVec, NonNull, RawMyVec}};
use crate::Sandboxable;
use chrono::naive::NaiveDateTime;

#[doc = "Library implementation of Sandboxable. Do not copy this docstring!"]
// TODO: (aportlan) T shouldn't have to be debuggable
impl<T: Sandboxable + Debug> Sandboxable for Vec<T> 
where T::InSandboxUnswizzled: Debug {
    type InSandboxUnswizzled = MyVecUnswizzled<T::InSandboxUnswizzled>;

    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled {
        // 1. Move everything in the vector into the sandbox
        let sandbox_vec = if T::is_identity() {
            // Fast memcpy method for if we don't have to unswizzle `T`
            let mut in_v: Vec<T, SandboxAllocator> = Vec::with_capacity_in(outside.len(), alloc.clone());

            // `Vec::to_raw_parts` could be used below but that was much slower. 
            // I think it forced the compiler to make an additional copy when we folded the vector earlier to get real ownership
            let (in_ptr, _, in_cap, in_alloc) = unsafe {
                let ptr = &mut in_v as *mut Vec<T, SandboxAllocator> as *mut MyVec<T, SandboxAllocator>;
                ((*ptr).buf.ptr.pointer as *mut T, (*ptr).buf.cap, (*ptr).len, (*ptr).buf.alloc.clone())
            };
            let (out_ptr, out_len, _) = unsafe {
                let ptr = &outside as *const Vec<T> as *mut MyVec<T>;
                ((*ptr).buf.ptr.pointer, (*ptr).buf.cap, (*ptr).len)
            };

            // memcpy into it
            unsafe { std::ptr::copy(out_ptr, in_ptr, out_len); }

            // return it
            unsafe { Vec::from_raw_parts_in(in_ptr as *mut T::InSandboxUnswizzled, out_len, in_cap, in_alloc) }
        } else {
            // Slow map strategy for if we do have to unswizzle `T`
            let mut sandbox_vec = Vec::with_capacity_in(outside.len(), alloc.clone());
            outside.into_iter().map(|b|{
                Sandboxable::into_sandbox(b, alloc.clone())
            }).collect_into(&mut sandbox_vec);

            sandbox_vec
        };
        
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
        // TODO: (aportlan) could implement opt here too for copying out
        v.iter().map(|u| {
            Sandboxable::out_of_sandbox(u, sandbox_ptr)
        }).collect::<Vec<T>>()
    }
}
