use std::ops::Deref;
use crate::{FastTransfer, SandboxInstance};

#[cfg(not(target_arch = "wasm32"))]
use crate::pointers::{ApplicationPtr, SandboxPtr};

// Mimic memory layout of Box<T, Global> so we can cast between and access ptr.
pub struct SandboxedBox {
    pub ptr: u32,
    // pub alloc: Global,   // 0-byte
}

// Implement `FastTransfer` for boxes of `FastTransfer` types.
#[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
impl<T: FastTransfer> FastTransfer for Box<T> {
    #[cfg(not(target_arch = "wasm32"))]
    type TypeInSandbox = SandboxedBox;

    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(outside: Self, sandbox: SandboxInstance) -> Self::TypeInSandbox {
        // Unswizzle the pointed-to data, and copy its stack portion to this variable.
        let outside = T::into_sandbox(*outside, sandbox);

        // Put the stack portion of the value on the heap of the sandbox, and get a Box to it.
        let b = Box::new_in(outside, sandbox);

        // Unswizzle the top-level pointer.
        let ptr = Box::into_raw(b);
        SandboxedBox {
            ptr: ApplicationPtr::new(ptr).unswizzle(sandbox).addr(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(inside: &Self::TypeInSandbox, sandbox: SandboxInstance) -> Self {
        // Swizzle the top-level pointer.
        let ptr: *mut T::TypeInSandbox = SandboxPtr::new(inside.ptr).swizzle(sandbox).ptr();
        let b = unsafe { Box::from_raw_in(ptr, sandbox) };

        // Now we have a valid pointer to the data, but the data remains in sandbox memory,
        // and may require swizzling recursively.
        Box::new(T::out_of_sandbox(b.deref(), sandbox))
    }
}
