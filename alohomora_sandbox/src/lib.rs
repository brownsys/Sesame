pub extern crate bincode;
pub extern crate serde;
pub extern crate serde_json;


use std::ffi::c_uint;
use serde::{Serialize, Deserialize};

// Used inside the sandbox for serializing/deserializing arguments and results.
#[cfg(target_arch = "wasm32")]
pub fn sandbox_preamble<'a, T: Deserialize<'a>, R: Serialize, F: Fn(T) -> R>(
    functor: F, arg: *const u8, len: u32) -> *mut u8 {
    use std::slice;
    use std::mem;

    // Deserialize input.
    let bytes = unsafe { slice::from_raw_parts(arg, len as usize) };
    let arg = bincode::deserialize(bytes).unwrap();

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
pub trait AlohomoraSandbox<'a, 'b, T: Serialize + Deserialize<'a>, R: Serialize + Deserialize<'b>> {
    fn invoke(arg: T) -> FinalSandboxOut<R>;
}

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


// Called by Alohomora (from the application process) to invoke the sandbox.
#[macro_export]
macro_rules! invoke_sandbox {
    ($functor:ident, $arg:ident) => {
        // Serialize argument.
        let v: Vec<u8> = ::alohomora_sandbox::bincode::serialize(&$arg).unwrap();
        // let arg = ::alohomora_sandbox::serde_json::to_string(&$arg).unwrap();
        // let arg = ::std::ffi::CString::new(arg).unwrap();

        // Invoke sandbox via C.
        let ret2: ::alohomora_sandbox::sandbox_out = unsafe { $functor(v.as_ptr(), v.len() as u32) };
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
