use crate::fast_transfer::vec::SandboxedVec;
use crate::{FastTransfer, SandboxInstance};

// Mimic memory layout of String so we can cast between and access ptr.
#[repr(transparent)]
pub struct SandboxedString {
    vec: SandboxedVec,
}

#[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
impl FastTransfer for String {
    #[cfg(not(target_arch = "wasm32"))]
    type TypeInSandbox = SandboxedString;

    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(outside: Self, sandbox: SandboxInstance) -> Self::TypeInSandbox {
        SandboxedString { vec: FastTransfer::into_sandbox(outside.into_bytes(), sandbox) }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(inside: &Self::TypeInSandbox, sandbox: SandboxInstance) -> Self {
        unsafe { String::from_utf8_unchecked(FastTransfer::out_of_sandbox(&inside.vec, sandbox)) }
    }
}
