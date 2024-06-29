use std::marker::PhantomData;

// The sandbox pointer type. T represents what the pointer points to (for helpful type-checking)
#[derive(Debug, Clone, Copy)]
pub struct SandboxPointer<T> {
    pub ptr: u32,                   // actual 4 byte pointer
    _phantom: PhantomData<T>        // holds the type info for T
}

impl<T> SandboxPointer<T> {
    pub fn new(ptr: u32) -> Self {
        SandboxPointer { ptr, _phantom: PhantomData::default() }
    }
}

const TOP32: u64 = 0xFFFFFFFF00000000;
const BOT32: u64 = 0xFFFFFFFF;

// convert a sandbox pointer to one that will work globally
pub fn swizzle_ptr<T, U>(ptr: &SandboxPointer<T>, known_ptr: *mut U) -> *mut T {
    let known_ptr = known_ptr as *mut std::ffi::c_void;
    
    let example_ptr: u64 = known_ptr as u64;
    let base: u64 = example_ptr & TOP32;
    let swizzled: u64 = (ptr.ptr as u64) + base;
    return swizzled as *mut T;
}

// convert global pointer to one that will work inside the sandbox
pub fn unswizzle_ptr<T>(ptr: *mut T) -> SandboxPointer<T> {
    let ptr = ptr as u64;
    let swizzled: u64 = ptr & BOT32;
    return SandboxPointer::<T>::new(swizzled as u32);
}