#[cfg(not(target_arch = "wasm32"))]
use crate::SandboxInstance;

mod primitives;
mod vec;
mod tuples;
mod r#box;
mod string;

/// Trait for directly copying to and from sandboxed memory without the need for serialization.
/// This is typically significantly faster.
pub trait FastTransfer {
    /// An equivalent struct with the memory layout our WASM sandbox expects for `Self`.
    /// (i.e. 32b with unswizzled pointers)
    #[cfg(not(target_arch = "wasm32"))]
    type TypeInSandbox;

    /// Deeply move object `outside` into sandbox memory & recursively swizzle it.
    /// General approach for this takes two steps:
    ///     1) recursively move everything this type points to into sandboxed memory
    ///     2) then (un)swizzle this type's stack data (to be boxed and passed into sandbox)
    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(outside: Self, sandbox: SandboxInstance) -> Self::TypeInSandbox;

    /// Deeply copy `inside` out of sandbox memory.
    /// General approach is in the opposite order of `into_sandbox`:
    ///     1) swizzle out this type's stack data
    ///     2) then recursively move everything it points to out of the sandbox
    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(inside: &Self::TypeInSandbox, sandbox: SandboxInstance) -> Self;
}

/// Marker trait, if T implements this, then it is a FastTransfer and the T::TypeInSandbox is T.
/// This means that data of type T needs only to be copied into the sandbox memory region, but
/// do not require swizzling or unswizzling, because they have identical memory layout
/// in both architectures and contain no pointers.
/// Be careful when implementing this for T where *alignment* might be different.
pub trait IdentityFastTransfer : Clone {}