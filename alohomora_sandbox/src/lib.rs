#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(vec_into_raw_parts)]
#![feature(specialization)]

pub extern crate bincode;
pub extern crate serde;
pub extern crate serde_json;

use std::any::Any;

use alloc::SandboxAllocator;
use serde::{Serialize, Deserialize};

pub mod ptr;
pub mod vec;
pub mod vec_impl;
pub mod str_impl;
pub mod prim_impl;
pub mod gen_impl;
pub mod alloc;
pub mod swizzle;

// Used inside the sandbox for serializing/deserializing arguments and results.
#[cfg(target_arch = "wasm32")]
pub fn sandbox_preamble<'a, T: std::fmt::Debug, R: Serialize, F: Fn(T) -> R>(
    functor: F, arg: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    use std::os::raw::c_void;
    use std::slice;
    use std::mem;

    // Convert arg to a pointer of the right type.
    let arg_ptr = arg as *mut T;
    
    let ret = unsafe { 
        // Put it into a box so we can get ownership
        // let b = Box::from_raw(ptr);
        let arg_val = SuperSandboxable::data_from_ptr(arg_ptr);
        
        // Call the actual function
        functor(*b)
    };

    // Serialize output.
    // println!("ret is {:?}", ret);
    let mut ret = bincode::serialize(&ret).unwrap();
    // println!("bincode ret is {:?}", ret);
    let size = ret.len() as u16;
    let size_1 = (size / 100) as u8;
    let size_2 = (size % 100) as u8;
    let mut vec2 = Vec::with_capacity(ret.len() + 2);
    vec2.push(size_1);
    vec2.push(size_2);
    for x in ret {
        vec2.push(x);
    }
    // println!("in ser w bytes {:?}", vec2);
    let ptttr = vec2.as_mut_ptr();
    mem::forget(vec2);
    ptttr as *mut std::ffi::c_void
}

// Trait that sandboxed functions should implement.
pub trait AlohomoraSandbox<'a, 'b, T, R> 
    where 
        T: SuperSandboxable,
        R: Serialize + Deserialize<'b>
{
    fn invoke(arg: *mut T::PointerRepresentation, sandbox_index: usize) -> R;
}

/// New mega trait that handles copying to/from sandboxes & all swizzling.
pub trait Sandboxable {
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

/// New super (even more mega) trait for both fast (`Sandboxable`) & slow (`Serializable`) path sandbox types
pub trait SuperSandboxable {
    /// the representation this points to in ptr form
    type PointerRepresentation; 
    // so for serializing, it'll point to a vec of u8
    // for 
    
    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> *mut Self::PointerRepresentation;
    //                                                      FIXME:    ^^thinking this should be c_void could be mistake

    // all run IN THE SANDBOX (so will automatically use the `InSandboxUnswizzled` version of the data)
    // just by virtue of being in the sandbox
    fn data_from_ptr(ptr: *mut Self::PointerRepresentation) -> Self;
    fn ptr_from_data(data: Self) -> *mut Self::PointerRepresentation;

    fn out_of_sandbox(ptr: *mut Self::PointerRepresentation) -> Self;
    
    //        [application]       ||   [sandbox]
    //                            ||
    //   *data* -> into_sandbox -------> *ptr*
    //    (64B)                   ||       |
    //                            ||  data_from_ptr
    //                            ||       |
    //                            ||    *data* (32B) <-> operate on in sandbox
    //                            ||       |
    //                            ||  ptr_from_data
    //                            ||       |
    // *data* <- out_of_sandbox <------- *ptr*
    //  (64B)                     ||

}

// impl<T> SuperSandboxable for T
//     where T: Serialize {
//         default type PointerRepresentation = Vec<u8>;
//         default fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> *mut Self::PointerRepresentation {
//             println!("serialize version");
//             todo!()
//         }


//         default fn data_from_ptr(ptr: *mut Self::PointerRepresentation) -> Self {
//             todo!()
//         }
//         default fn ptr_from_data(data: Self) -> *mut Self::PointerRepresentation {
//             todo!()
//         }

//         default fn out_of_sandbox(ptr: *mut Self::PointerRepresentation) -> Self {
//             todo!()
//         }
// }

impl<T: Sandboxable> SuperSandboxable for T {
    type PointerRepresentation = Self;
    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> *mut Self::PointerRepresentation {
        let val = Sandboxable::into_sandbox(outside, alloc.clone());
        let b = Box::new_in(val, alloc);
        Box::into_raw(b) as *mut Self::PointerRepresentation
    }

    fn data_from_ptr(ptr: *mut Self) -> Self {
        // Take value from box
        unsafe{ *Box::from_raw(ptr as *mut T) }
    }
    fn ptr_from_data(data: Self) -> *mut Self {
        // Put the output into a box
        let b = Box::new(data);

        // Pass on the ptr
        Box::into_raw(b)
    }

    fn out_of_sandbox(ptr: *mut Self::PointerRepresentation) -> Self {
        // Move returned values out of the sandbox & swizzle.
        let ret_val = unsafe{ Box::leak(Box::from_raw(ptr as *mut <Self as Sandboxable>::InSandboxUnswizzled)) };
        let result = Sandboxable::out_of_sandbox(ret_val, ptr as usize);
        result
    }
}

// fn process<T>(item: T)
// where
//     T: Serialize, // Ensure T can use the slow method
// {
//     // Check if T also implements the fast method
//     if let Some(sandboxable) = (&item as &dyn Any).downcast_ref::<dyn Sandboxable>() {
//         let p = Sandboxable::into_sandbox(sandboxable, SandboxAllocator::new(0));
//         // sandboxable.process_fast();
//         println!("sandboxabled");
//     } else {
//         let s = serde_json::to_string(&item).unwrap();
//         println!("serialized w {}", s);
//     }
// }

// This should be generated by a macro.
#[cfg(not(target_arch = "wasm32"))]
extern "C" {
    pub fn invoke_free_c(arg1: *mut u8);
}

#[cfg(not(target_arch = "wasm32"))]
#[repr(C)]
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
        // ^^the line above is now already done by the SuperSandboxable::into_sandbox()
        
        // Invoke sandbox via C.
        let ret2: ::alohomora_sandbox::sandbox_out = unsafe { $functor($arg as *mut std::ffi::c_void, $sandbox_index) };
        let ret = ret2.result;

        // println!("ret 2 {:?}", ret2);
        // println!("ret {:?}", ret);

        let bytes = unsafe {std::slice::from_raw_parts(ret, ret2.size as usize)};
        // println!("before deser w bytes {:?}", bytes);
        let result = ::alohomora_sandbox::bincode::deserialize(bytes).unwrap();
        // println!("after deser");

        // Return.
        return result;
    }
}