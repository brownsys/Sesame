use std::{alloc::{AllocError, Allocator}, ptr::{slice_from_raw_parts_mut, NonNull}};

#[derive(Debug, Clone)]
pub struct SandboxAllocator {
    pub sandbox_index: usize, // the index of the sandbox to allocate in
}

impl SandboxAllocator {
    pub fn new(sandbox_index: usize) -> Self { SandboxAllocator { sandbox_index } }
}

unsafe impl Allocator for SandboxAllocator {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        // allocate in sandbox memory
        // println!("rust-- trying to allocate with allocator {:?} for layout {:?}", self, layout);
        // Allocate memory in sandbox.
        let raw_ptr = unsafe { crate::alloc_mem_in_sandbox(layout.size(), self.sandbox_index) };
        
        // Convert to fat pointer.
        let thin_ptr = raw_ptr as *mut u8;
        let fat_ptr = slice_from_raw_parts_mut(thin_ptr, layout.size());

        match NonNull::new(fat_ptr) {
            Some(nn) => Ok(nn),
            None => Err(AllocError),
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        // Free in sandbox memory.
        println!("rust-- trying to deallocate with allocator {:?}, ptr {:?} and layout {:?}", self, ptr, layout);
        unsafe {
            crate::free_mem_in_sandbox(ptr.as_ptr() as *mut std::ffi::c_void, self.sandbox_index);
        }
    }
}

pub trait AllocateableInSandbox {
    type UsingSandboxAllocator;
    // creates a new allocation in the sandbox using `alloc` with the same structure as `info`
    unsafe fn allocate_in_sandbox(info: *mut Self, alloc: SandboxAllocator) -> *mut Self::UsingSandboxAllocator;
}

impl<T> AllocateableInSandbox for Vec<T> {
    type UsingSandboxAllocator = Vec<T, SandboxAllocator>;
    unsafe fn allocate_in_sandbox(info: *mut Self, alloc: SandboxAllocator) -> *mut Self::UsingSandboxAllocator {
        let v = Vec::with_capacity_in((*info).len(), alloc.clone());
        let b = Box::new_in(v, alloc);
        Box::into_raw(b)
    }
}