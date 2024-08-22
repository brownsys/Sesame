use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::SandboxAllocator, ptr::*, swizzle::*, vec::{MyVec, NonNull, RawMyVec}, FastSandboxTransfer};

// Implement for Strings
#[derive(Debug)]
pub struct StringUnswizzled {
    vec: MyVecUnswizzled<u8>,
}

#[doc = "Library implementation of FastSandboxTransfer. Do not copy this docstring!"]
impl FastSandboxTransfer for String {
    type InSandboxUnswizzled = StringUnswizzled;
    fn into_sandbox(outside: Self, alloc: crate::alloc::SandboxAllocator) -> Self::InSandboxUnswizzled {
        let vec = outside.bytes().collect::<Vec<u8>>();

        StringUnswizzled{
            vec: FastSandboxTransfer::into_sandbox(vec, alloc),
        }
    }

    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, sandbox_ptr: usize) -> Self where Self: Sized {
        let vec = FastSandboxTransfer::out_of_sandbox(&inside.vec, sandbox_ptr);
        String::from_utf8(vec).unwrap()
    }
}