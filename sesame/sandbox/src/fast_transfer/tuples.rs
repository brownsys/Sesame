use crate::{IdentityFastTransfer, FastTransfer, SandboxInstance};

macro_rules! sandboxable_tuple_impl {
    ($([$A:tt,$i:tt]),*) => {
        #[doc = "Library implementation of FastTransfer. Do not copy this docstring!"]
        impl<$($A,)*> FastTransfer for ($($A,)*) where $($A: FastTransfer,)* {
            #[cfg(not(target_arch = "wasm32"))]
            type TypeInSandbox = ($($A::TypeInSandbox,)*);

            #[cfg(not(target_arch = "wasm32"))]
            default fn into_sandbox(outside: Self, sandbox: SandboxInstance) -> ($($A::TypeInSandbox,)*) {
                ($($A::into_sandbox(outside.$i, sandbox),)*)
            }

            #[cfg(not(target_arch = "wasm32"))]
            default fn out_of_sandbox(inside: &($($A::TypeInSandbox,)*), sandbox: SandboxInstance) -> Self {
                ($($A::out_of_sandbox(&inside.$i, sandbox),)*)
            }
        }
        
        #[doc = "Library implementation of IdentityFastTransfer. Do not copy this docstring!"]
        impl<$($A,)*> IdentityFastTransfer for ($($A,)*) where $($A: IdentityFastTransfer + FastTransfer,)* {}
    };
}

sandboxable_tuple_impl!([T1, 0]);
sandboxable_tuple_impl!([T1, 0], [T2, 1]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2], [T4, 3]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2], [T4, 3], [T5, 4]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2], [T4, 3], [T5, 4], [T6, 5]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2], [T4, 3], [T5, 4], [T6, 5], [T7, 6]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2], [T4, 3], [T5, 4], [T6, 5], [T7, 6], [T8, 7]);
