use std::any;

use crate::FastSandboxTransfer;

// Implement `FastSandboxTransfer` for tuples of `FastSandboxTransfer` types.
macro_rules! sandboxable_tuple_impl {
    ($([$A:tt,$i:tt]),*) => {
        #[doc = "Library implementation of FastSandboxTransfer. Do not copy this docstring!"]
        impl<$($A,)*> FastSandboxTransfer for ($($A,)*) where 
            $($A: FastSandboxTransfer,)* {
            type InSandboxUnswizzled = ($($A::InSandboxUnswizzled,)*);

            // This tuple will be an identity iff all of its values are identities
            fn is_identity() -> bool {
                let b = ($($A::is_identity() &&)* true);
                return b;
            }

            fn into_sandbox(outside: Self, alloc: crate::alloc::SandboxAllocator) -> Self::InSandboxUnswizzled {
                ($(FastSandboxTransfer::into_sandbox(outside.$i, alloc.clone()),)*)
            }
            fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, any_sandbox_ptr: usize) -> Self {
                ($(FastSandboxTransfer::out_of_sandbox(&inside.$i, any_sandbox_ptr),)*)
            }
        }
    };
}
sandboxable_tuple_impl!([T1, 0]);
sandboxable_tuple_impl!([T1, 0], [T2, 1]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2], [T4, 3]);
sandboxable_tuple_impl!([T1, 0], [T2, 1], [T3, 2], [T4, 3], [T5, 4]);