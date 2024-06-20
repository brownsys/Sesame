use std::{alloc::{AllocError, Allocator}, ptr::{slice_from_raw_parts_mut, NonNull}};

#[derive(Debug, Clone)]
/// An allocator for getting memory inside a sandbox from our sandbox pool.
pub struct SandboxAllocator {
    sandbox_index: usize, // the index of the sandbox to allocate in
}

impl SandboxAllocator {
    pub fn new(sandbox_index: usize) -> Self { SandboxAllocator { sandbox_index } }
    pub fn index(&self) -> usize { self.sandbox_index }
}

unsafe impl Allocator for SandboxAllocator {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
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

    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        // Free in sandbox memory. 
        // (For now we're doing nothing bc the sandboxed memory will be free when its reset anyway)

        // unsafe {
        //     crate::free_mem_in_sandbox(ptr.as_ptr() as *mut std::ffi::c_void, self.sandbox_index);
        // }
    }
}

/// Trait for types that are able to be allocated in a sandbox as the type `UsingSandboxAllocator`. 
/// (We use a different type bc they might have different allocator generics like Vec<T, SandboxAllocator> instead of Vec<T>)
pub trait AllocateableInSandbox {
    type UsingSandboxAllocator;
    /// Creates a new allocation in the sandbox using `alloc` with the same structure as `info`.
    fn allocate_in_sandbox(info: &Self, alloc: &SandboxAllocator) -> Self::UsingSandboxAllocator;

    // /// Converts the sandbox allocation `inside` to a raw pointer fully contained in the sandbox for use in the functor.
    // unsafe fn to_raw_ptr(inside: Self::UsingSandboxAllocator) -> *mut Self;
}

impl<T> AllocateableInSandbox for Vec<T> {
    type UsingSandboxAllocator = Vec<T, SandboxAllocator>;
    fn allocate_in_sandbox(info: &Self, alloc: &SandboxAllocator) -> Self::UsingSandboxAllocator {
        let v = Vec::with_capacity_in((*info).len(), alloc.clone());
        v
    }
}