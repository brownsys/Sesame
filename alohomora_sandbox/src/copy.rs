use std::convert::TryInto;

/// Trait for types that are able to be copied into sandbox allocations.
pub trait Copiable: crate::alloc::AllocateableInSandbox {
    /// Copies all the data from `old` into the sandbox allocated `new`.
    unsafe fn copy(new: &mut Self::UsingSandboxAllocator, old: &Self);
}

/// Trait for types that can be 'swizzled' --
///     all nested pointers are converted to internal sandbox pointers
///   & all data types are converted to their 32 bit equivalents
pub trait Swizzleable {
    type Unswizzled;
    /// (For going INTO sandboxes) 
    /// Unswizzles all of `inside`'s 64 bit pointers and converts all data types to their 32 bit equivalents.
    unsafe fn unswizzle(inside: Self) -> Self::Unswizzled;
    
    /// (For going OUT OF sandboxes) 
    /// Swizzles all of `inside`'s 32 bit pointers and converts all data types back to their 64 bit equivalents.
    /// 
    unsafe fn swizzle(inside: Self::Unswizzled) -> Self 
    where Self: Sized {
        todo!();
    }
}

/// Trait for all types that are swizzleable, but don't need to change when being swizzled.
/// (i.e. anything with no pointers, references or pointer sized types like `usize`)
pub trait SwizzleableIdentity {
    unsafe fn unswizzle(inside: Self) -> Self where Self: Sized{ inside }
    unsafe fn swizzle(inside: Self) -> Self where Self: Sized{ inside }
}


impl Swizzleable for (u64, (), u64) {
    type Unswizzled = (u64, (), u64);
    unsafe fn unswizzle(inside: Self) -> Self::Unswizzled {
        inside
    }
}
  
impl Swizzleable for (usize, (), usize) {
    type Unswizzled = (u32, (), u32);
    unsafe fn unswizzle(inside: Self) -> Self::Unswizzled {
        (inside.0.try_into().unwrap(), (), inside.2.try_into().unwrap())
    }
    unsafe fn swizzle(inside: Self::Unswizzled) -> Self 
        where Self: Sized {
        (inside.0.try_into().unwrap(), (), inside.2.try_into().unwrap())
    }
}