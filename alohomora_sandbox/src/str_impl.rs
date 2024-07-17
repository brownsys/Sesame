use crate::{copy::Swizzleable, vec_impl::{MyVecUnswizzled, Sandboxable}};

pub struct StringUnswizzled {
    vec: MyVecUnswizzled<u8>,
}

impl Swizzleable for String {
    type Unswizzled = StringUnswizzled;
    unsafe fn unswizzle(inside: Self) -> Self::Unswizzled {
        let inside_vec = inside.as_bytes().to_owned();
        StringUnswizzled {
            vec: Swizzleable::unswizzle(inside_vec),
        }
    }

    unsafe fn swizzle(inside: Self::Unswizzled) -> Self 
        where Self: Sized {
        let inside_vec = Swizzleable::swizzle(inside.vec);
        String::from_utf8(inside_vec).unwrap()
    }
}

impl Sandboxable for String {
    type InSandboxUnswizzled = StringUnswizzled;
    fn into_sandbox(outside: Self, alloc: crate::alloc::SandboxAllocator) -> Self::InSandboxUnswizzled {
        let vec = outside.bytes().collect::<Vec<u8>>();
        StringUnswizzled{
            vec: Sandboxable::into_sandbox(vec, alloc),
        }
    }

    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, sandbox_ptr: usize) -> Self where Self: Sized {
        let vec = Sandboxable::out_of_sandbox(&inside.vec, sandbox_ptr);
        String::from_utf8(vec).unwrap()
    }
}