pub extern crate bincode;
pub extern crate serde;
pub extern crate serde_json;

use std::{ffi::c_uint, os::raw::c_void};
use serde::{Serialize, Deserialize};

// Used inside the sandbox for serializing/deserializing arguments and results.
#[cfg(target_arch = "wasm32")]
pub fn sandbox_preamble<'a, R: Serialize, F: Fn(*mut std::ffi::c_void) -> R>(
    functor: F, arg: *mut std::ffi::c_void, len: u32) -> *mut u8 {
    use std::slice;
    use std::mem;

    // Deserialize input.
    // let bytes = unsafe { slice::from_raw_parts(arg, len as usize) };
    // let arg = bincode::deserialize(bytes).unwrap();

    // Call function.
    let ret = functor(arg);

    // Serialize output.
    let mut ret = bincode::serialize(&ret).unwrap();
    let size = ret.len() as u16;
    let size_1 = (size / 100) as u8;
    let size_2 = (size % 100) as u8;
    let mut vec2 = Vec::with_capacity(ret.len() + 2);
    vec2.push(size_1);
    vec2.push(size_2);
    for x in ret {
        vec2.push(x);
    }
    let ptttr = vec2.as_mut_ptr();
    mem::forget(vec2);
    ptttr
}

// Trait that sandboxed functions should implement.
pub trait AlohomoraSandbox<'a, 'b, T, R: Serialize + Deserialize<'b>> {
    fn invoke(arg: T) -> FinalSandboxOut<R>;
}
// pub trait AlohomoraSandbox2<'a, 'b, T: Serialize + Deserialize<'a>, R: Serialize + Deserialize<'b>> {
//     fn invoke(arg: T) -> FinalSandboxOut<R>;
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

// take a pointer to the vector and add all the shit
// pub extern "C" fn transfer_arg(arg: Box<Vec<(f64, u64)>>, sandbox_loc: &mut Box<Vec<(f64, u64)>>) -> bool {
// #[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn alloc_mem_in_sandbox(size: usize, sandbox: usize) -> *mut std::ffi::c_void;
}

// 1. swizzle it out
// 2. do our modifications on the data type
// 3. make sure we swizzle back all pointers in the data type

// use the same example strategy from the RLBox paper to find example pointer
pub fn swizzle_ptr<T>(ptr: u32, known_ptr: *mut std::ffi::c_void) -> *mut T {
    let top32: u64 = 0xFFFFFFFF00000000;
    let bot32: u32 = 0xFFFFFFFF;
    let example_ptr: u64 = known_ptr as u64;
    let base: u64 = example_ptr & top32;
    let swizzled: u64 = (ptr as u64) + base;
    return swizzled as *mut T;
}

pub fn unswizzle_ptr<T>(ptr: *mut T) -> u32 {
    let top32: u64 = 0xFFFFFFFF00000000;
    let bot32: u64 = 0xFFFFFFFF;
    let ptr = ptr as u64;
    let swizzled: u64 = ptr & bot32;
    return swizzled as u32;
}

// Called by Alohomora (from the application process) to invoke the sandbox.
#[macro_export]
macro_rules! invoke_sandbox {
    ($functor:ident, $arg:ident) => {
        // Serialize argument.
        // let v: Vec<u8> = ::alohomora_sandbox::bincode::serialize(&$arg).unwrap();
        // let arg = ::alohomora_sandbox::serde_json::to_string(&$arg).unwrap();
        // let arg = ::std::ffi::CString::new(arg).unwrap();
        // let input_vec: *mut Vec<(f64, u64)> = $arg as *mut Vec<(f64, u64)>;

        let ptr: *mut std::ffi::c_void = unsafe {
            // TODO: handle sandbox allocation
            ::alohomora_sandbox::alloc_mem_in_sandbox(10, 0)
        };

        println!("***the arg is $arg {:?}", $arg);
 
        let inside = ptr as *mut GrandparentUnswizzled;
        let outside = $arg as *mut Grandparent;

        unsafe {
            println!("original struct (in sandbox) is {:?}", (&*inside));
            println!("outside (in sandbox) is {:?}", (&*outside));

            // let real_ptr = ptr as *mut MyVecUnswizzled;
            // println!("len of sandbox struct is {:?}", (*real_ptr).len);

            // let mut swizzled: *mut Grandparent = nested::swizzle_grand(real_ptr);

            // println!("swizzled struct is {:?}", *swizzled);
            // println!("swizzled favorite kid is {:?}", *(*swizzled).favorite_kid);
            // println!("their favorite kid is {:?}", *(*(*swizzled).favorite_kid).favorite_kid);
            
            println!("unswizzling it (so changes are reflected in sandbox)");

            let new = nested::Swizzleable::unswizzle(outside, inside);
            // println!("new is {:?} with original {:?}", new, real_ptr);
            // swizzled.unswizzle();

            println!("inside is now {:?}", (&*inside));
            // println!("swizzled favorite kid is {:?}", *(*real_ptr).favorite_kid);
            // println!("their favorite kid is {:?}", *(*(*real_ptr).favorite_kid).favorite_kid);
        }

        // Invoke sandbox via C.
        println!("*entering FUNCTOR");

        let ret2: ::alohomora_sandbox::sandbox_out = unsafe { $functor(ptr, 0) };

        println!("*just finished some macro business");
        let ret = ret2.result;

        // Deserialize output.
        let bytes = unsafe {std::slice::from_raw_parts(ret, ret2.size as usize)};
        let result = ::alohomora_sandbox::bincode::deserialize(bytes).unwrap();

        // Free memory.
        unsafe { ::alohomora_sandbox::invoke_free_c(ret) };

        // Return.
        return ::alohomora_sandbox::FinalSandboxOut { result: result, size: ret2.size, setup: ret2.setup, teardown: ret2.teardown };
    }
}
