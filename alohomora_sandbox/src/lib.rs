#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(vec_into_raw_parts)]
#![feature(specialization)]

pub extern crate bincode;
pub extern crate serde;
pub extern crate serde_json;

use std::{convert::TryInto, fmt::Debug, mem};

use alloc::SandboxAllocator;
use ptr::{swizzle_ptr, SandboxPointer};
use serde::{Serialize, Deserialize};

pub mod ptr;
pub mod vec;
// pub mod vec_impl;
// pub mod str_impl;
pub mod prim_impl;
pub mod gen_impl;
pub mod alloc;
pub mod swizzle;

// Used inside the sandbox for serializing/deserializing arguments and results.
#[cfg(target_arch = "wasm32")]
pub fn sandbox_preamble<'a, T: SuperSandboxable, R: SuperSandboxable, F: Fn(T) -> R>(
    functor: F, arg_ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    // use std::os::raw::c_void;
    use std::slice;
    use std::mem;

    // Convert arg to a pointer of the right type.
    // let arg_ptr = arg as *mut c_void;
    
    let ret = unsafe {
        // Put it into a box so we can get ownership
        // let b = Box::from_raw(ptr);
        let arg_val: T = SuperSandboxable::data_from_ptr(arg_ptr);
        
        // Call the actual function
        functor(arg_val)
    };

    // println!("ret in preamble is {:?}", ret);

    // Serialize output.
    // println!("ret is {:?}", ret);
    let p = SuperSandboxable::ptr_from_data(ret);
    println!("ptr in preamble is {:p}", p);
    p
}

// Trait that sandboxed functions should implement.
pub trait AlohomoraSandbox<'a, 'b, T, R> 
    where 
        T: SuperSandboxable,
        R: SuperSandboxable
{
    fn invoke(arg: *mut std::ffi::c_void, sandbox_index: usize) -> R;
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
    // so for serializing, it'll point to a vec of u8
    // for 
    
    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> *mut std::ffi::c_void;
    //                                                      FIXME:    ^^thinking this should be c_void could be mistake

    // all run IN THE SANDBOX (so will automatically use the `InSandboxUnswizzled` version of the data)
    // just by virtue of being in the sandbox
    fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self;
    fn ptr_from_data(data: Self) -> *mut std::ffi::c_void;

    fn out_of_sandbox(ptr: *mut std::ffi::c_void) -> Self;
    
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

impl<'a, T: Serialize + Deserialize<'a> + Debug> SuperSandboxable for T {
        default fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> *mut std::ffi::c_void {
            // need to serialize into the sandbox
            println!("into sandbox serialize path");
            let v: Vec<u8> = bincode::serialize(&outside).unwrap();

            let mut vec_in = Vec::with_capacity_in(v.len(), alloc.clone());
            for c in v {
                vec_in.push(c);
            }
            let (ptr, len, _) = vec_in.into_raw_parts();
            
            // the *mut will be 4B instead of 8B in the sandbox
            // but thats okay bc the alignment is 8B from the u64 so the extra 4B of padding will be added automatically
            let tup: (*mut u8, u64) = (ptr, len as u64);
            let b = Box::new_in(tup, alloc);
            Box::into_raw(b) as *mut std::ffi::c_void
        }

        default fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            println!("data_from_ptr serialize path");
            let real_ptr = ptr as *mut (*mut u8, u64);
            let b = unsafe { Box::from_raw(real_ptr) };
            
            let tup = *b;
            let (ptr, len) = tup;
            let bytes = unsafe { std::slice::from_raw_parts(ptr, len.try_into().unwrap()) };
            let val: Self = bincode::deserialize(&bytes).unwrap();
            val
        }
        default fn ptr_from_data(data: Self) -> *mut std::ffi::c_void {
            println!("serialize ptr from data");

            // need to serialize into the sandbox
            let v: Vec<u8> = bincode::serialize(&data).unwrap();

            let (ptr, len, _) = v.into_raw_parts();
            let tup: (*mut u8, u64) = (ptr, len as u64);
            // println!("have tup {:?}", tup);
            // println!("size of tup {:?}", std::mem::size_of_val(&tup));
            let b = Box::new(tup);
            // println!("have box {:?}", b);
            // println!("size of box {:?}", std::mem::size_of_val(&b));
            let p = Box::into_raw(b) as *mut std::ffi::c_void;
            println!("\tfinal ptr_from_data {:p}", p);
            p
        }

        default fn out_of_sandbox(ptr: *mut std::ffi::c_void) -> Self {
            println!("initial out_of_sandbox {:p}", ptr);
            let real_ptr = ptr as *mut (u32, u64);
            let b = unsafe { Box::leak(Box::from_raw(real_ptr)) };

            // println!("have box {:?}", b);
            // println!("size of box {:?}", std::mem::size_of_val(&b));
            
            let tup = *b;
            // println!("have tup {:?}", tup);
            // println!("size of tup {:?}", std::mem::size_of_val(&tup));
            let (ptr_unswiz, len) = tup;
            
            // TODO: swizzz
            let ptr_swiz: *mut u8 = swizzle_ptr(&SandboxPointer::new(ptr_unswiz), ptr as usize);
            let bytes = unsafe { std::slice::from_raw_parts(ptr_swiz as *const u8, len.try_into().unwrap()) };
            let val: Self = bincode::deserialize(&bytes).unwrap();

            println!("\tfinal val {:?}", val);
            val
        }
}

impl<'a, T: Sandboxable + Serialize + Deserialize<'a> + Debug> SuperSandboxable for T {
    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> *mut std::ffi::c_void {
        println!("sandboxable version");
        let val = Sandboxable::into_sandbox(outside, alloc.clone());
        let b = Box::new_in(val, alloc);
        Box::into_raw(b) as *mut std::ffi::c_void
    }

    fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        // Take value from box
        println!("sandbox data_from for type {}", std::any::type_name::<Self>());
        unsafe{ *Box::from_raw(ptr as *mut T) }
    }
    fn ptr_from_data(data: Self) -> *mut std::ffi::c_void {
        // Put the output into a box
        // todo!();
        println!("sandboxable ptr_from_data");
        println!("\tdata is {:?}", data);
        let b = Box::new(data);

        // Pass on the ptr
        let p = Box::into_raw(b) as *mut std::ffi::c_void;
        println!("\tptr is {:p}", p);
        p
    }

    fn out_of_sandbox(ptr: *mut std::ffi::c_void) -> Self {
        // todo!();
        println!("sandboxable out_of_sandbox");
        println!("\tptr is {:p}", ptr);
        // Move returned values out of the sandbox & swizzle.
        let ret_val = unsafe{ Box::leak(Box::from_raw(ptr as *mut <Self as Sandboxable>::InSandboxUnswizzled)) };
        let result = Sandboxable::out_of_sandbox(ret_val, ptr as usize);
        println!("\tfinal val is {:?}", result);
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
#[derive(Debug)]
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
        let ret2: ::alohomora_sandbox::sandbox_out = 
            unsafe { $functor($arg as *mut std::ffi::c_void, $sandbox_index) };

        let ret = ret2.result; // the result struct isn't used so we can modify it

        println!("ret 2 {:?}", ret2);
        println!("ret {:?}", ret);

        let result: $ret_ty = 
            <$ret_ty as ::alohomora_sandbox::SuperSandboxable>
                ::out_of_sandbox(ret as *mut std::ffi::c_void);

        // Return.
        return result;
    }
}