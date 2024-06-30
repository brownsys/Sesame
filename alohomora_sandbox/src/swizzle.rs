// trait for being able to copy instance into sandboxed memory
pub trait Swizzleable {
    type Unswizzled;
    unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled, old_inside: &Self) -> *mut Self::Unswizzled;
    // unsafe fn swizzle(inside: *mut Self::Unswizzled, outside: *mut Self) -> *mut Self;
}