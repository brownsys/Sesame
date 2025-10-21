use std::convert::TryInto;
use chrono::NaiveDateTime;
use crate::{IdentityFastTransfer, FastTransfer, SandboxInstance};

// Implement `FastSandboxTransfer` for primitives that won't change in the sandbox.
macro_rules! derive_sandboxable_identity {
    ($t:ty) => {
        #[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
        impl FastTransfer for $t {
            #[cfg(not(target_arch = "wasm32"))]
            type TypeInSandbox = $t;

            #[cfg(not(target_arch = "wasm32"))]
            fn into_sandbox(outside: Self, _: SandboxInstance) -> Self::TypeInSandbox {
                outside
            }

            #[cfg(not(target_arch = "wasm32"))]
            fn out_of_sandbox(inside: &Self::TypeInSandbox, _: SandboxInstance) -> Self {
                (*inside).clone()
            }
        }

        #[doc = "Library implementation of IdentityFastTransfer. Do not copy this docstring!"]
        impl IdentityFastTransfer for $t {}
    }
}

derive_sandboxable_identity!(());
derive_sandboxable_identity!(bool);
derive_sandboxable_identity!(u8);
derive_sandboxable_identity!(u16);
derive_sandboxable_identity!(u32);
derive_sandboxable_identity!(u64);
derive_sandboxable_identity!(i8);
derive_sandboxable_identity!(i16);
derive_sandboxable_identity!(i32);
derive_sandboxable_identity!(i64);
derive_sandboxable_identity!(f32);
derive_sandboxable_identity!(f64);
derive_sandboxable_identity!(NaiveDateTime);

#[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
impl FastTransfer for usize {
    #[cfg(not(target_arch = "wasm32"))]
    type TypeInSandbox = u32;

    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(outside: Self, _: SandboxInstance) -> u32 { outside.try_into().unwrap() }

    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(inside: &u32, _: SandboxInstance) -> Self { inside.clone().try_into().unwrap() }
}

#[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
impl FastTransfer for isize {
    #[cfg(not(target_arch = "wasm32"))]
    type TypeInSandbox = i32;

    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(outside: Self, _: SandboxInstance) -> i32 { outside.try_into().unwrap() }

    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(inside: &i32, _: SandboxInstance) -> Self { inside.clone().try_into().unwrap() }
}
