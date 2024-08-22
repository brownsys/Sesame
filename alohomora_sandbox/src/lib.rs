#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(vec_into_raw_parts)]
#![feature(specialization)]

pub extern crate bincode;
pub extern crate serde;
pub extern crate serde_json;

use std::{convert::TryInto, fmt::Debug};

use alloc::SandboxAllocator;
use ptr::{swizzle_ptr, SandboxPointer};
use serde::{Serialize, Deserialize};

pub mod ptr;
pub mod vec;
pub mod alloc;
pub mod swizzle;
pub mod transfer;

// Used inside the sandbox for serializing/deserializing arguments and results.
#[cfg(target_arch = "wasm32")]
pub fn sandbox_preamble<'a, T: SandboxTransfer, R: SandboxTransfer, F: Fn(T) -> R>(
    functor: F, arg_ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    use std::slice;
    use std::mem;

    // Convert arg to a pointer of the right type.
    // let arg_ptr = arg as *mut c_void;
    
    let ret = unsafe {
        let arg_val: T = SandboxTransfer::data_from_ptr(arg_ptr);
        
        // Call the actual function
        functor(arg_val)
    };

    // Serialize output.
    let p = SandboxTransfer::ptr_from_data(ret);
    println!("ptr in preamble is {:p}", p);
    p
}

// Trait that sandboxed functions should implement.
pub trait AlohomoraSandbox<'a, 'b, T, R> 
    where 
        T: SandboxTransfer,
        R: SandboxTransfer
{
    fn invoke(arg: *mut std::ffi::c_void, sandbox_index: usize) -> R;
}

/// Trait for directly copying to & from sandboxed memory without the need for serialization.
/// This is typically significantly faster.
pub trait FastSandboxTransfer {
    /// An equivalent struct with the memory layout our WASM sandbox will expect for `Self`.
    /// (i.e. 32b with unswizzled pointers)
    type InSandboxUnswizzled;

    /// Returns true for a type `T` if and only if `T::InSandboxUnswizzled` is identical to `T`.
    fn is_identity() -> bool { false }

    /// Deeply move object `outside` into sandbox memory & recursively swizzle it.
    /// General approach for this takes two steps: 
    ///     1) recursively move everything this type points to into sandboxed memory
    ///     2) then (un)swizzle this type's stack data (to be boxed and passed into sandbox)
    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled;

    /// Deeply copy `inside` out of sandbox memory.
    /// General approach is in the opposite order of `into_sandbox`:
    ///     1) swizzle out this type's stack data
    ///     2) then recursively move everything it points to out of the sandbox
    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, any_sandbox_ptr: usize) -> Self;
}

/// Trait for transferring data into & out of Alohomora Sandboxes. 
/// Automatically implemented for all types satisfying `serde::Serialize + serde::Deserialize` or `FastSandboxTransfer`.
/// Deriving `FastSandboxTransfer` for custom structs will likely increase performance.
pub trait SandboxTransfer {
    // Function that converts a type to 32 bits, moves it fully into the sandbox,
    // and returns a pointer to it.
    fn into_sandbox(data: Self, alloc: SandboxAllocator) -> *mut std::ffi::c_void;

    // Function that takes a pointer to a 32 bit sandbox type, 
    // recursively unswizzles & converts it to 64 bits, and returns the 64 type.
    fn out_of_sandbox(ptr: *mut std::ffi::c_void) -> Self;


    // Runs in sandboxes to reconstruct a 32 bit version of `Self` from a pointer.
    fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self;
    // Runs in sandboxes to convert a 32 bit version of `Self` to a pointer.
    fn ptr_from_data(data: Self) -> *mut std::ffi::c_void;

    
    //        [APPLICATION]         ||    [SANDBOX]
    //                              ||
    // *data* --> into_sandbox() ---------> *ptr*
    //  [64b]                       ||        |
    //                              ||  data_from_ptr()
    //                              ||        |
    //                              ||     *data* [32b] <-> operate on in sandbox
    //                              ||        |
    //                              ||  ptr_from_data()
    //                              ||        |
    // *data* <- out_of_sandbox() <-------- *ptr*
    //  [64b]                       ||
}

impl<'a, T: Serialize + Deserialize<'a> > SandboxTransfer for T {
        default fn into_sandbox(data: Self, alloc: SandboxAllocator) -> *mut std::ffi::c_void {
            // Serialize `data` into bytes
            println!("into sandbox serialize path");
            let v: Vec<u8> = bincode::serialize(&data).unwrap();

            // Move those bytes into the sandbox
            let mut vec_in = Vec::with_capacity_in(v.len(), alloc.clone());
            for c in v {
                vec_in.push(c);
            }
            let (ptr, len, _) = vec_in.into_raw_parts();
            
            // Create custom fat ptr to store the addr & len of the serialized Vec
            let tup: (*mut u8, u64) = (ptr, len as u64);
            // NOTE: the *mut will be 4B instead of 8B in the sandbox
            //       but thats okay bc the alignment is 8B from the u64 so the extra 4B of padding will be added automatically
            // TODO: (aportlan) actually is this undefined behavior in rust?
            Box::into_raw(Box::new_in(tup, alloc)) as *mut std::ffi::c_void
        }

        default fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            println!("data_from_ptr serialize path");
            // Get addr & len of serialized vec
            let real_ptr = ptr as *mut (*mut u8, u64);
            let (ptr, len) = unsafe { *Box::from_raw(real_ptr) };

            // Rebuild slice from that addr & len
            let bytes = unsafe { std::slice::from_raw_parts(ptr, len.try_into().unwrap()) };

            // Deserialize
            bincode::deserialize(&bytes).unwrap()
        }
        default fn ptr_from_data(data: Self) -> *mut std::ffi::c_void {
            println!("serialize ptr from data");

            // Serialize `data` into bytes
            let v: Vec<u8> = bincode::serialize(&data).unwrap();

            // Store the addr & len of those bytes in a box
            let (ptr, len, _) = v.into_raw_parts();
            let tup: (*mut u8, u64) = (ptr, len as u64);

            Box::into_raw(Box::new(tup)) as *mut std::ffi::c_void
        }

        default fn out_of_sandbox(ptr: *mut std::ffi::c_void) -> Self {
            println!("initial out_of_sandbox {:p}", ptr);
            let real_ptr = ptr as *mut (u32, u64);
            let b = unsafe { Box::leak(Box::from_raw(real_ptr)) };
            let (ptr_unswiz, len) = *b;
            
            // Swizzle the ptr to the serialized bytes 
            // (since it was written in the sandbox it's unswizzled & 32b)
            let ptr_swiz: *mut u8 = swizzle_ptr(&SandboxPointer::new(ptr_unswiz), ptr as usize);
            
            // Reconstruct bytes slice from ptr & len
            let bytes = unsafe { std::slice::from_raw_parts(ptr_swiz as *const u8, len.try_into().unwrap()) };
            
            bincode::deserialize(&bytes).unwrap()
        }
}

impl<'a, T: FastSandboxTransfer + Serialize + Deserialize<'a>> SandboxTransfer for T {
    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> *mut std::ffi::c_void {
        println!("sandboxable version");
        // Move the value into sandboxed memory
        let val = FastSandboxTransfer::into_sandbox(outside, alloc.clone());

        // Put it into a box in the sandbox for passing as pointer
        Box::into_raw(Box::new_in(val, alloc)) as *mut std::ffi::c_void
    }

    fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        println!("sandbox data_from for type {}", std::any::type_name::<Self>());

        // Take value from box
        unsafe{ *Box::from_raw(ptr as *mut T) }
    }
    fn ptr_from_data(data: Self) -> *mut std::ffi::c_void {
        println!("sandboxable ptr_from_data");

        // Put value into box
        Box::into_raw(Box::new(data)) as *mut std::ffi::c_void
    }

    fn out_of_sandbox(ptr: *mut std::ffi::c_void) -> Self {
        println!("sandboxable out_of_sandbox");
        
        // Reconstruct the 32 bit type from a Box pointer
        let ret_val = unsafe{ Box::leak(Box::from_raw(ptr as *mut <Self as FastSandboxTransfer>::InSandboxUnswizzled)) };

        // Move returned values out of the sandbox & swizzle
        FastSandboxTransfer::out_of_sandbox(ret_val, ptr as usize)
    }
}

// This should be generated by a macro.
#[cfg(not(target_arch = "wasm32"))]
extern "C" {
    pub fn invoke_free_c(arg1: *mut u8);
}

#[cfg(not(target_arch = "wasm32"))]
#[repr(C)]
#[derive(Debug)]
// TODO: (aportlan) we only use the result now, so can remove this struct
pub struct sandbox_out {
    pub result: *mut u8,
    pub size: u32,
    pub setup: ::std::os::raw::c_ulonglong,
    pub teardown: ::std::os::raw::c_ulonglong,
}

pub struct FinalSandboxOut<R> {
    pub result: R,
    pub size: u32,
    pub setup: u64,
    pub teardown: u64,
}

// #[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn alloc_mem_in_sandbox(size: usize, sandbox: usize) -> *mut std::ffi::c_void;
    pub fn free_mem_in_sandbox(ptr: *mut std::ffi::c_void, sandbox: usize);
    pub fn get_lock_on_sandbox() -> usize;
    pub fn unlock_sandbox(index: usize);
}

// Called by Alohomora (from the application process) to invoke the sandbox.
#[macro_export]
macro_rules! invoke_sandbox {
    ($functor:ident, $arg:ident, $arg_ty:ty, $ret_ty:ty, $sandbox_index:ident) => {

        // `$arg` is already a swizzled 32 bit type for the sandbox, 
        // so we just make a raw pointer for passing through 'C land' 
        // then the preamble can reconstruct the real type back in Rust
        // let new_inside_ptr = Box::into_raw(Box::new_in($arg, ::alohomora_sandbox::alloc::SandboxAllocator::new($sandbox_index)));
        // ^^the line above is now already done by the SandboxTransfer::into_sandbox()
        
        // Invoke sandbox via C.
        let ret2: ::alohomora_sandbox::sandbox_out = 
            unsafe { $functor($arg as *mut std::ffi::c_void, $sandbox_index) };

        let ret = ret2.result; // the result struct isn't used so we can modify it

        println!("ret 2 {:?}", ret2);
        println!("ret {:?}", ret);

        let result: $ret_ty = 
            <$ret_ty as ::alohomora_sandbox::SandboxTransfer>
                ::out_of_sandbox(ret as *mut std::ffi::c_void);

        // Return.
        return result;
    }
}