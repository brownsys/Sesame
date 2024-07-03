use crate::alloc::AllocateableInSandbox;

/// Trait for types that are able to be copied into sandbox allocations.
pub trait Copiable: AllocateableInSandbox {
    /// Copies all the data from `old` into the sandbox allocated `new`.
    unsafe fn copy(new: &mut Self::UsingSandboxAllocator, old: &Self);
}