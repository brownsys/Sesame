use std::marker::PhantomData;
use crate::SandboxInstance;

// Rely on rlbox implementation for pointer swizzling/unswizzling.
// Essentially, these implementation subtract the start address of the sandbox memory from the
// pointer, and cast it to 32 bits.
extern "C" {
    fn get_sandbox_pointer(size: *mut std::ffi::c_void, sandbox: usize) -> u32;
    fn get_unsandboxed_pointer(ptr: u32, sandbox: usize) -> *mut std::ffi::c_void;
}

/// Logical representation of a pointer in the host application architecture, usually 64 bits.
#[repr(transparent)]
pub struct ApplicationPtr<T> {
    ptr: *mut T,
}
impl<T> ApplicationPtr<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }
    pub fn ptr(&self) -> *mut T {
        self.ptr
    }
    pub fn unswizzle(&self, sandbox: SandboxInstance) -> SandboxPtr<T> {
        let addr = self.ptr() as *mut std::ffi::c_void;
        let addr = unsafe { get_sandbox_pointer(addr, sandbox.index()) };
        SandboxPtr::new(addr)
    }
}

/// Logical representation of a pointer in the sandbox architecture, 32 bits (with some offset).
#[repr(transparent)]
pub struct SandboxPtr<T> {
    ptr: u32,
    data: PhantomData<T>
}
impl<T> SandboxPtr<T> {
    pub fn new(ptr: u32) -> Self {
        Self { ptr, data: PhantomData }
    }
    pub fn addr(&self) -> u32 {
        self.ptr
    }
    pub fn swizzle(&self, sandbox: SandboxInstance) -> ApplicationPtr<T> {
        let addr = unsafe { get_unsandboxed_pointer(self.addr(), sandbox.index()) };
        ApplicationPtr::new(addr as *mut T)
    }
}