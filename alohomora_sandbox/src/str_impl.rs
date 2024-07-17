use crate::{copy::Swizzleable, vec_impl::MyVecUnswizzled};

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