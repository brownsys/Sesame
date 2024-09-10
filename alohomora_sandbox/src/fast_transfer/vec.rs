use crate::{IdentityFastTransfer, FastTransfer, SandboxInstance};

#[cfg(not(target_arch = "wasm32"))]
use crate::pointers::{ApplicationPtr, SandboxPtr};

// This is an identical memory layout to how Vec<T> is internally on the sandbox 32bit arch.
// We use this to unswizzle/swizzle the insides of the vector.
// Alignment is fine since all fields are 4bytes and thus 4bytes aligned on both
// 32bit and 64bit archs.
pub struct SandboxedVec {
    pub(self) buf: SandboxedRawVec,
    pub(self) len: u32,
}

struct SandboxedRawVec {
    pub(self) ptr: SandboxedNonNull,
    pub(self) cap: u32,
    // pub alloc: Global,   // 0-byte
}

#[repr(transparent)]
struct SandboxedNonNull {
    pub(self) pointer: u32,
}

/// FastTransfer for vectors containing complex swizzled types.
#[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
impl<T: FastTransfer + Sized> FastTransfer for Vec<T> {
    #[cfg(not(target_arch = "wasm32"))]
    type TypeInSandbox = SandboxedVec;

    #[cfg(not(target_arch = "wasm32"))]
    default fn into_sandbox(outside: Self, sandbox: SandboxInstance) -> Self::TypeInSandbox {
        // Copy everything inside the vector into the sandbox and unswizzle it.
        let mut sandbox_vec = Vec::with_capacity_in(outside.len(), sandbox);
        outside.into_iter().map(|b| T::into_sandbox(b, sandbox)).collect_into(&mut sandbox_vec);

        // Now, we only need to swizzle the pointer within the sandbox vec, and change sizes
        // of len and cap. The actual heap content of the vec is all done!
        let (ptr, len, cap, _) = sandbox_vec.into_raw_parts_with_alloc();
        let ptr = ApplicationPtr::new(ptr).unswizzle(sandbox).addr();
        let len = usize::into_sandbox(len, sandbox);
        let cap = usize::into_sandbox(cap, sandbox);

        // Return the unswizzled complete vec. Note that the vec ptr, len, and cap are
        // members of this struct and thus stack allocated! The surrounding code needs
        // to copy this stack portion of the vector into sandbox memory separately.
        // However, the heap portion of the vec is all ready.
        SandboxedVec {
            buf: SandboxedRawVec {
                ptr: SandboxedNonNull {
                    pointer: ptr
                },
                cap
            },
            len: len
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    default fn out_of_sandbox(inside: &Self::TypeInSandbox, sandbox: SandboxInstance) -> Self {
        // Extract the stack-components of the vector and swizzle them.
        let (ptr, len, cap) = (inside.buf.ptr.pointer, inside.len, inside.buf.cap);
        let ptr: *mut T::TypeInSandbox = SandboxPtr::new(ptr).swizzle(sandbox).ptr();
        let len = usize::out_of_sandbox(&len, sandbox);
        let cap = usize::out_of_sandbox(&cap, sandbox);

        // This is a valid vector, but its data is in the sandbox memory, and any nested
        // pointers or structs are not yet swizzled.
        let vec = unsafe {
            Vec::from_raw_parts_in(ptr, len, cap, sandbox)
        };

        // Swizzle and copy elements out of sandbox.
        vec.iter().map(|t: &T::TypeInSandbox| T::out_of_sandbox(t, sandbox)).collect()
    }
}

/// FastTransfer for vectors containing simple non-swizzled types.
#[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
impl<T: IdentityFastTransfer + FastTransfer + Sized> FastTransfer for Vec<T> {
    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(outside: Self, sandbox: SandboxInstance) -> Self::TypeInSandbox {
        // Copy everything inside the vector into the sandbox.
        // Because T is IdentityFastTransfer, we do not need to swizzle the elements.
        let mut sandbox_vec: Vec<T, SandboxInstance> = Vec::with_capacity_in(outside.len(), sandbox);

        // Gets compiled down to some memory bound checks followed by a memcpy.
        sandbox_vec.extend_from_slice(&outside);

        // Now, we only need to swizzle the pointer within the sandbox vec, and change sizes
        // of len and cap. The actual heap content of the vec is all done!
        let (ptr, len, cap, _) = sandbox_vec.into_raw_parts_with_alloc();
        let ptr = ApplicationPtr::new(ptr).unswizzle(sandbox).addr();
        let len = usize::into_sandbox(len, sandbox);
        let cap = usize::into_sandbox(cap, sandbox);

        // Return the unswizzled complete vec. Note that the vec ptr, len, and cap are
        // members of this struct and thus stack allocated! The surrounding code needs
        // to copy this stack portion of the vector into sandbox memory separately.
        // However, the heap portion of the vec is all ready.
        SandboxedVec {
            buf: SandboxedRawVec {
                ptr: SandboxedNonNull {
                    pointer: ptr,
                },
                cap
            },
            len
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(inside: &Self::TypeInSandbox, sandbox: SandboxInstance) -> Self {
        // Extract the stack-components of the vector and swizzle them.
        let (ptr, len, cap) = (inside.buf.ptr.pointer, inside.len, inside.buf.cap);
        let ptr: *mut T = SandboxPtr::new(ptr).swizzle(sandbox).ptr();  // T = T::TypeInSandbox
        let len = usize::out_of_sandbox(&len, sandbox);
        let cap = usize::out_of_sandbox(&cap, sandbox);

        // This is a valid vector, but its data is in the sandbox memory, and any nested
        // pointers or structs are not yet swizzled.
        let vec = unsafe {
            Vec::from_raw_parts_in(ptr, len, cap, sandbox)
        };

        // Because T is IdentityFastTransfer, we do not need to swizzle the elements.
        // We just need to copy them!
        let mut output: Vec<T> = Vec::with_capacity(vec.len());

        // Gets compiled down to some memory bound checks followed by a memcpy.
        output.extend_from_slice(&vec);

        output
    }
}
