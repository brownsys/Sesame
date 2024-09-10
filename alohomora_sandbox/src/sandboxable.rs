use std::ops::Deref;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{FastTransfer, SandboxInstance};

#[cfg(not(target_arch = "wasm32"))]
use crate::pointers::{ApplicationPtr, SandboxPtr};

/// Trait for transferring data into & out of Alohomora Sandboxes.
/// Automatically implemented for all types satisfying `serde::Serialize + serde::DeserializeOwned`
/// or `FastSandboxTransfer`.
/// Deriving `FastSandboxTransfer` for custom structs will likely increase performance.
pub trait SandboxableType {
    // Function that converts a type to 32 bits, moves it fully into the sandbox,
    // and returns a pointer to it.
    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(data: Self, sandbox: SandboxInstance) -> *mut std::ffi::c_void;

    // Function that takes a pointer to a 32 bit sandbox type,
    // recursively unswizzles & converts it to 64 bits, and returns the 64 type.
    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(ptr: *mut std::ffi::c_void, sandbox: SandboxInstance) -> Self;

    // Runs in sandboxes to reconstruct a 32 bit version of `Self` from a pointer.
    #[cfg(target_arch = "wasm32")]
    fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self;

    // Runs in sandboxes to convert a 32 bit version of `Self` to a pointer.
    #[cfg(target_arch = "wasm32")]
    fn ptr_from_data(data: Self) -> *mut std::ffi::c_void;

    //        [APPLICATION]             ||                       [SANDBOX]
    //   ============================   ||   ===================================================
    //     arg compatible with [64b]    ||
    //   ptr1[64b] = into_sandbox(arg)  ||
    //           ptr1[64b]   ---- auto-unswizz by rlbox -->      ptr1[32b]
    //                                  ||
    //                                  ||                arg = data_from_ptr(ptr1)
    //                                  ||                arg compatible with [32b]
    //                                  ||
    //                                  ||              ptr2[32b] = ptr_from_data(ret)
    //           ptr2[64b]   <--- auto-swizz by rlbox ---        ptr2[32b]
    //                                  ||
    //    ret <- out_of_sandbox(ptr2)   ||
    //     ret compatible with [64b]    ||
    //
    // into_sandbox(arg) is responsible for unswizzling what's inside arg, so that *ptr1 is [32b].
    // Only the top-level pointer ptr1 is [64b].
    //
    // out_of_sandbox(ptr2) is responsible for swizzling what's inside ret, so that ret is [64b].
    // While the top-level pointer ptr2 is [64b], it points to data in [32b].
    //
}


/// Implement SandboxableType for any type that implements serde's Serialize and DeserializeOwned
/// by using bincode serialization.
impl<T: Serialize + DeserializeOwned> SandboxableType for T {
    #[cfg(not(target_arch = "wasm32"))]
    default fn into_sandbox(data: Self, sandbox: SandboxInstance) -> *mut std::ffi::c_void {
        // Serialize `data` into bytes
        let v: Vec<u8> = bincode::serialize(&data).unwrap();

        // Move those bytes into the sandbox
        let mut vec_in = Vec::with_capacity_in(v.len(), sandbox);
        for c in v {
            vec_in.push(c);
        }
        let (ptr, len, cap) = vec_in.into_raw_parts();
        let ptr = ApplicationPtr::new(ptr).unswizzle(sandbox).addr();

        // Create custom fat ptr to store the addr & len of the serialized Vec
        let tup: (u32, u64, u64) = (ptr, len as u64, cap as u64);
        Box::into_raw(Box::new_in(tup, sandbox)) as *mut std::ffi::c_void
    }

    #[cfg(not(target_arch = "wasm32"))]
    default fn out_of_sandbox(ptr: *mut std::ffi::c_void, sandbox: SandboxInstance) -> Self {
        let ptr = ptr as *mut (u32, u64, u64);
        let (ptr, len, cap) = *unsafe { Box::from_raw_in(ptr, sandbox) };

        // Swizzle the ptr to the serialized bytes
        // (since it was written in the sandbox it's unswizzled and in 32 bits)
        let ptr = SandboxPtr::new(ptr).swizzle(sandbox).ptr();

        // Reconstruct vec from ptr len and capacity.
        let vec = unsafe {
            Vec::from_raw_parts_in(ptr, len as usize, cap as usize, sandbox)
        };
        bincode::deserialize(&vec).unwrap()
    }

    #[cfg(target_arch = "wasm32")]
    default fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        // Get addr, len, and capacity of serialized vec
        let ptr = ptr as *mut (u32, u64, u64);
        let (ptr, len, cap) = *unsafe { Box::from_raw(ptr) };
        let ptr = ptr as *mut u8;

        // Reconstruct vec from ptr len and capacity.
        let vec = unsafe { Vec::from_raw_parts(ptr, len as usize, cap as usize) };
        bincode::deserialize(&vec).unwrap()
    }

    #[cfg(target_arch = "wasm32")]
    default fn ptr_from_data(data: Self) -> *mut std::ffi::c_void {
        // Serialize `data` into bytes
        let v: Vec<u8> = bincode::serialize(&data).unwrap();

        // Store the addr, len, and capacity of those bytes in a box
        let (ptr, len, cap) = v.into_raw_parts();
        let tup: (*mut u8, u64, u64) = (ptr, len as u64, cap as u64);

        Box::into_raw(Box::new(tup)) as *mut std::ffi::c_void
    }
}

#[cfg(not(target_arch = "wasm32"))]
type TypeInSandbox<T> = <T as FastTransfer>::TypeInSandbox;

/// Specialize SandboxableType implementation for types that also implement FastTransfer
/// Requires nightly `specialization` feature.
/// Relies on deep copying the data to/from sandbox and swizzling/unswizzling all nested pointers.
impl<T: FastTransfer + Serialize + DeserializeOwned> SandboxableType for T {
    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(outside: Self, sandbox: SandboxInstance) -> *mut std::ffi::c_void {
        // (Deep) copy the value into sandboxed memory
        let val: TypeInSandbox<T> = FastTransfer::into_sandbox(outside, sandbox);

        // Put it into a box in the sandbox for passing as pointer
        Box::into_raw(Box::new_in(val, sandbox)) as *mut std::ffi::c_void
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(ptr: *mut std::ffi::c_void, sandbox: SandboxInstance) -> Self {
        // Reconstruct the 32 bit type from a Box pointer
        let ptr: *mut TypeInSandbox<T> = ptr as *mut TypeInSandbox<T>;
        let ret: Box<TypeInSandbox<T>, SandboxInstance> = unsafe { Box::from_raw_in(ptr, sandbox) };

        // Move returned values out of the sandbox & swizzle
        FastTransfer::out_of_sandbox(ret.deref(), sandbox)
    }

    #[cfg(target_arch = "wasm32")]
    fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        // This is ok: T::TypeInSandbox <=> T within the sandbox arch.
        let ptr = ptr as *mut T;
        *unsafe { Box::from_raw(ptr) }
    }

    #[cfg(target_arch = "wasm32")]
    fn ptr_from_data(data: Self) -> *mut std::ffi::c_void {
        // Put value into box
        Box::into_raw(Box::new(data)) as *mut std::ffi::c_void
    }
}
