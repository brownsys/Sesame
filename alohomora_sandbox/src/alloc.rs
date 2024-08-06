use std::{alloc::{AllocError, Allocator}, ptr::{slice_from_raw_parts_mut, NonNull}};

#[derive(Debug, Clone)]
/// An allocator for getting memory inside a sandbox from our sandbox pool.
pub struct SandboxAllocator {
    sandbox_index: usize, // the index of the sandbox to allocate in
}

impl SandboxAllocator {
    // TODO: (aportlan) should be private
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