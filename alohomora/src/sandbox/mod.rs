use std::{fmt::Debug, result};

use alohomora_sandbox::{alloc::SandboxAllocator, unlock_sandbox, FastSandboxTransfer, SandboxTransfer};
use serde::{Serialize, Deserialize};

use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;

// Expose alohomora_sandbox API that controls the interface outside sandbox.
pub use alohomora_sandbox::{AlohomoraSandbox, FinalSandboxOut};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::AlohomoraSandbox;

pub struct SandboxInstance {
    sandbox_index: usize,
    alloc: SandboxAllocator,
}

impl SandboxInstance {
    /// Create new sandbox instance. (in reality just get one from the pool)
    pub fn new() -> Self {
        let sandbox_index = unsafe{ ::alohomora_sandbox::get_lock_on_sandbox() };
        SandboxInstance { sandbox_index, alloc: SandboxAllocator::new(sandbox_index) }
    }

    /// An allocator to allocate into this instance's sandbox.
    pub fn alloc(&self) -> SandboxAllocator {
        self.alloc.clone()
    }

    /// Copies `t` into a sandbox and executes the specified function on it.
    pub fn copy_and_execute<'a, 'b, S, T, R>(t: T) -> BBox<R, AnyPolicy>
        where
            T: AlohomoraType,
            T::Out: SandboxTransfer,
            R: SandboxTransfer,
            S: AlohomoraSandbox<'a, 'b, T::Out, R>,
    {
        // Remove boxes from args.
        let outer_boxed = fold::<AnyPolicy, _, _>(t).unwrap();
        let (t, p) = outer_boxed.consume();

        // Create a new sandbox instance.
        let instance = SandboxInstance::new();

        // move the arg into the sandbox and conver it to a ptr
        let arg_ptr: *mut std::ffi::c_void = SandboxTransfer::into_sandbox(t, instance.alloc());

        // Pass that ptr to the function.
        let result = S::invoke(arg_ptr, instance.sandbox_index);

        BBox::new(result, p)
    }

    // Executes `S` on variable `t` assuming that `t` is already in sandboxed memory 
    // (i.e. it has been allocated with this `SandboxInstance`'s `SandboxAllocator`).
    // pub fn execute<'a, 'b, S, T, R, A>(self, t: T) -> BBox<R, AnyPolicy>
    //     where
                                         
    //         T: AlohomoraType<AnyPolicy, SandboxAllocator>,                      //     To ensure `T` is Alohomora type and using a sandbox allocator
    //         A: AllocateableInSandbox + Swizzleable,                             // <--
    //         T::Out: Into<<A as AllocateableInSandbox>::UsingSandboxAllocator>,  // <-| To ensure `T::Out` uses a sandbox allocator
    //         <A as AllocateableInSandbox>::UsingSandboxAllocator: Swizzleable,
    //         A::Unswizzled: From<<<A as AllocateableInSandbox>::UsingSandboxAllocator as Swizzleable>::Unswizzled>, // can't remember why we need this
    //         R: Swizzleable,
    //         S: AlohomoraSandbox<'a, 'b, A, R>,
    // {
    //     let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
    //     // 1. remove bboxes.
    //     let outer_boxed = fold(t).unwrap();
    //     let (t, p) = outer_boxed.consume();
    //     let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
    //     println!("execute - folding took {:?}", end - start);

    //     let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
    //     // 2. unswizzle data type.
    //     let sandbox_alloc: <A as AllocateableInSandbox>::UsingSandboxAllocator = t.into();
    //     let final_arg = unsafe { Swizzleable::unswizzle(sandbox_alloc).into() };
    //     let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
    //     println!("execute - unswizzling took {:?}", end - start);

    //     let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
    //     // 3. call functor.
    //     let ret = S::invoke(final_arg, self.sandbox_index);
    //     let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
    //     println!("execute - calling functor took {:?}", end - start);

    //     let ret = unsafe{ Box::from_raw(ret) };
    //     let result = unsafe{ Swizzleable::swizzle(*ret) };

    //     BBox::new(result, p)
    // }
}

impl Drop for SandboxInstance {
    fn drop(&mut self) {
        // Unlock sandbox mutex when this goes out of scope.
        unsafe{ unlock_sandbox(self.sandbox_index); }
    }
}