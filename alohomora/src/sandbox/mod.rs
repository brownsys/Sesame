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

/// Copies `t` into a sandbox and executes the specified function on it.
pub fn execute_sandbox<'a, 'b, S, T, R>(t: T) -> BBox<R, AnyPolicy>
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

// TODO: (aportlan) this struct is pretty redundant now, so could probably be removed
// and its functionality offloaded to the allocator
pub(crate) struct SandboxInstance {
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
}

impl Drop for SandboxInstance {
    fn drop(&mut self) {
        // Unlock sandbox mutex when this goes out of scope.
        unsafe{ unlock_sandbox(self.sandbox_index); }
    }
}