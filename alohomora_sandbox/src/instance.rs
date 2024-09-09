// FFI declarations for managing sandbox pool and sandboxes memory.
#[cfg(not(target_arch = "wasm32"))]
extern "C" {
    fn alloc_mem_in_sandbox(size: usize, sandbox: usize) -> *mut std::ffi::c_void;
    fn free_mem_in_sandbox(ptr: *mut std::ffi::c_void, sandbox: usize);
    fn get_lock_on_sandbox() -> usize;
    fn unlock_sandbox(sandbox: usize);
}

/// A locked sandbox instance from the sandbox pool.
/// Also is an allocator for getting memory inside a sandbox from our sandbox pool.
#[derive(Debug, Clone, Copy)]
pub struct SandboxInstance {
    sandbox_index: usize, // the index of the sandbox to allocate in
}
#[cfg(not(target_arch = "wasm32"))]
impl SandboxInstance {
    pub(crate) fn new() -> Self {
        let sandbox_index = unsafe { get_lock_on_sandbox() };
        SandboxInstance { sandbox_index }
    }
    pub(crate) fn index(&self) -> usize {
        self.sandbox_index
    }
    pub(crate) fn release(self) {
        unsafe { unlock_sandbox(self.sandbox_index) }
    }
}
#[cfg(not(target_arch = "wasm32"))]
unsafe impl std::alloc::Allocator for SandboxInstance {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        // Allocate memory in sandbox.
        let raw_ptr = unsafe { alloc_mem_in_sandbox(layout.size(), self.sandbox_index) };

        // Convert to fat pointer.
        let thin_ptr = raw_ptr as *mut u8;
        let fat_ptr = std::ptr::slice_from_raw_parts_mut(thin_ptr, layout.size());

        match std::ptr::NonNull::new(fat_ptr) {
            Some(nn) => Ok(nn),
            None => Err(std::alloc::AllocError),
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        // Free in sandbox memory.
        unsafe { free_mem_in_sandbox(ptr.as_ptr() as *mut std::ffi::c_void, self.sandbox_index); }
    }
}
