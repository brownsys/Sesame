use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::SandboxAllocator, ptr::*, swizzle::*, vec::{MyVec, NonNull, RawMyVec}};
use crate::Sandboxable;
use chrono::naive::NaiveDateTime;

impl<T: Sandboxable + Debug> Sandboxable for Vec<T> 
where T::InSandboxUnswizzled: Debug {
    type InSandboxUnswizzled = MyVecUnswizzled<T::InSandboxUnswizzled>;

    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled {
        let sandbox_vec = if T::is_identity() {
            // let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
            // make sandbox in sandbox
            let in_v: Vec<T, SandboxAllocator> = Vec::with_capacity_in(outside.len(), alloc.clone());
            let (in_ptr, _, in_cap, in_alloc) = in_v.into_raw_parts_with_alloc();
            let (out_ptr, out_len, _) = outside.into_raw_parts();

            // memcpy into it
            unsafe { std::ptr::copy(out_ptr, in_ptr, out_len); }

            // return it
            let v = unsafe {Vec::from_raw_parts_in(in_ptr as *mut T::InSandboxUnswizzled, out_len, in_cap, in_alloc)};
            // the transmute of `in_ptr` is okay bc the fact that this is an identity guaruntees that 
            // `T` is the same as `T::InSandbox Unswizzled`
            // let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
            // let first = end - start;
            // println!("\tvec into sandbox - first part (fast identity) took {first}");
            v
        } else {
            // 1. move everything inside to sandbox
            // let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
            let mut sandbox_vec = Vec::with_capacity_in(outside.len(), alloc.clone()); // TODO: with capacity here
            outside.into_iter().map(|b|{
                Sandboxable::into_sandbox(b, alloc.clone())
            }).collect_into(&mut sandbox_vec);

            // let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
            // let first = end - start;
            // println!("\tvec into sandbox - first part (non identity) took {first}");
            sandbox_vec
        };
        

        // 1b. convert to myvec so we can access private members
        // let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        let ptr: *const Vec<T::InSandboxUnswizzled, SandboxAllocator> = &sandbox_vec as *const Vec<T::InSandboxUnswizzled, SandboxAllocator>;
        let ptr = ptr as *mut MyVec<T::InSandboxUnswizzled, SandboxAllocator>;
        let a = unsafe { (*ptr).clone() };

        // 2. swizzle our metadata on the stack
        let ret = unswizzle_myvec(a);

        // let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // let first = end - start;
        // println!("\tvec into sandbox - second part took {first}");
        ret
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
