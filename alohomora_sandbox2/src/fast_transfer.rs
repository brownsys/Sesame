#[cfg(not(target_arch = "wasm32"))]
use crate::SandboxInstance;

/// Trait for directly copying to and from sandboxed memory without the need for serialization.
/// This is typically significantly faster.
pub trait FastSandboxTransfer {
    /// An equivalent struct with the memory layout our WASM sandbox expects for `Self`.
    /// (i.e. 32b with unswizzled pointers)
    type TypeInSandbox;

    /// Returns true for a type `T` if and only if `T::SandboxedType` is identical to `T`.
    #[cfg(not(target_arch = "wasm32"))]
    fn is_identity() -> bool { false }

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

