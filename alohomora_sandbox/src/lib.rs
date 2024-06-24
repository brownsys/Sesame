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
    let TOP32: u64 = 0xFFFFFFFF00000000;
    let BOT32: u32 = 0xFFFFFFFF;
    let example_ptr: u64 = known_ptr as u64;
    println!("example ptr is {example_ptr}");
    let base: u64 = example_ptr & TOP32;
    println!("base is {example_ptr}");
    let swizzled: u64 = (ptr as u64) + base;
    return swizzled as *mut T;
}

pub fn unswizzle_ptr<T>(ptr: *mut T) -> u32 {
    let TOP32: u64 = 0xFFFFFFFF00000000;
    let BOT32: u64 = 0xFFFFFFFF;
    let ptr = ptr as u64;
    let swizzled: u64 = ptr & BOT32;
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
        println!("in macro");
        let ptr: *mut std::ffi::c_void = unsafe {
            let ptr = ::alohomora_sandbox::alloc_mem_in_sandbox(10, 0);
            println!("ptr1 is {:?}", ptr);
            ptr
        };

        #[derive(Debug)]
        pub struct TestStructUnswizzled {
            my_int: u32, // 4
            my_float: f32, // 4
            my_float2: f64, // 8 <- 16 total
            ptr_to_buddy: u32, // 8b
        }

        #[derive(Debug)]
        pub struct TestStructReal {
            _unswizzled: *mut TestStructUnswizzled,
            my_int: u32,
            my_float: f32,  
            my_float2: f64,
            ptr_to_buddy: *mut i32,
        }

        pub unsafe fn swizzle(unswizzled: *mut TestStructUnswizzled) -> TestStructReal {
            TestStructReal{
                _unswizzled: unswizzled,
                my_int: (*unswizzled).my_int,
                my_float: (*unswizzled).my_float,
                my_float2: (*unswizzled).my_float2,
                ptr_to_buddy: ::alohomora_sandbox::swizzle_ptr((*unswizzled).ptr_to_buddy, unswizzled as *mut c_void)
            }
        }
        
        impl TestStructReal{
            pub unsafe fn unswizzle(&self) {
                (*self._unswizzled).my_int = self.my_int;
                (*self._unswizzled).my_float = self.my_float;
                (*self._unswizzled).my_float2 = self.my_float2;
                (*self._unswizzled).ptr_to_buddy = ::alohomora_sandbox::unswizzle_ptr(self.ptr_to_buddy);
            }
        }
        
        println!("macro got ptr {:?}", ptr);

        let real_ptr = ptr as *mut TestStructUnswizzled;

        println!("macro got real ptr {:?}", real_ptr);

        unsafe {
            println!("**original struct is {:?}", (&*real_ptr));
            let mut swizzled = swizzle(real_ptr);
            println!("swizzled {:?}", swizzled);
            
            println!("my buddy is {:?}", *(swizzled.ptr_to_buddy));
            swizzled.my_int += 100000;
            *(swizzled.ptr_to_buddy) = 1000;

            swizzled.unswizzle();
            println!("post unswizzle original is {:?}", (&*real_ptr));
            println!("my buddy is {:?}", *(swizzled.ptr_to_buddy));
        }
        
        
        unsafe {
            // println!("macro real ptr deref {:?}", *real_ptr);

            let mut b: Box<TestStructUnswizzled> = Box::from_raw(real_ptr);
            // println!("macro | real ptr is {:?}", real_ptr);
            // println!("macro | box is {:?}", b);

            // b.my_int = 21;
            
            // println!("macro | w capacity {:?}", (*b).capacity());
            // println!("macro | box is w interior {:?}", *b);

            // println!("macro | leaking box (bc we didnt allocate it)");
            Box::leak(b); // leak the box bc we didn't really allocate it
            // (*b).push((0.1, 3));
            // TODO: change for push_within_capacity() to make sure we're not exceeding sandbox allocated mem
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
