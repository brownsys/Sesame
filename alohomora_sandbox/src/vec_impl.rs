use std::fmt::Debug;
use crate::{alloc::SandboxAllocator, copy::{Copiable, Swizzleable}, ptr::*};

impl<T: Debug> Swizzleable for Vec<T, SandboxAllocator> {
    type Unswizzled = MyVecUnswizzled<T>;
    unsafe fn unswizzle(inside: Self) -> MyVecUnswizzled<T> {
        let ptr = (&inside as *const Vec<T, SandboxAllocator>) as *const crate::vec::MyVec<T, SandboxAllocator>;
        let old_inside = (*ptr).clone();

        MyVecUnswizzled {
            buf: RawMyVecUnswizzled{
                ptr: NonNullUnswizzled{pointer: unswizzle_ptr(old_inside.buf.ptr.pointer as *mut T)},
                cap: old_inside.buf.cap as u32,
            },
            len: old_inside.len as u32,
        }
    }
}

impl <T> Swizzleable for Vec<T> {
    type Unswizzled = MyVecUnswizzled<T>;
    unsafe fn unswizzle(_inside: Self) -> MyVecUnswizzled<T> {
        // shouldn't ever actually use this bc we should only use sandbox allocated vecs
        todo!();
    }
}

impl<T: Debug> Copiable for Vec<T> {
    unsafe fn copy(new: &mut Self::UsingSandboxAllocator, old: &Self) {
        let new_ptr = (new as *mut Vec<T, SandboxAllocator>) as *mut crate::vec::MyVec<T, SandboxAllocator>;
        let old_ptr = (old as *const Vec<T>) as *const crate::vec::MyVec<T>;
        
        std::ptr::copy((*old_ptr).buf.ptr.pointer, (*new_ptr).buf.ptr.pointer as *mut T, (*new_ptr).buf.cap);
        (*new_ptr).len = (*old_ptr).len;
    }
}

#[derive(Debug)]
pub struct MyVecUnswizzled<T> {
    pub buf: RawMyVecUnswizzled<T>,
    pub len: u32,
}

#[derive(Debug)]
pub struct RawMyVecUnswizzled<T> {
    pub ptr: NonNullUnswizzled<T>,
    pub cap: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct NonNullUnswizzled<T> {
    pub pointer: SandboxPointer<T>,
}

pub unsafe fn unswizzle_raw_myvec<T>(myvec: &crate::vec::RawMyVec<T>) -> RawMyVecUnswizzled<T>{
    let old_ptr = myvec.ptr.pointer;
    let old_cap = myvec.cap;

    RawMyVecUnswizzled{
        ptr: NonNullUnswizzled{pointer: unswizzle_ptr(old_ptr as *mut T)},
        cap: old_cap as u32
    }
}

pub unsafe fn unswizzle_myvec<T>(myvec: crate::vec::MyVec<T>) -> MyVecUnswizzled<T> {
    let old_len = myvec.len;
    let old_raw = &myvec.buf;
    MyVecUnswizzled{
        buf: unswizzle_raw_myvec(old_raw),
        len: old_len as u32,
    }
}