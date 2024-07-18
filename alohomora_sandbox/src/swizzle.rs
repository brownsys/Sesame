use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::SandboxAllocator, ptr::*, vec::{MyVec, NonNull, RawMyVec}};
use chrono::naive::NaiveDateTime;

// TODO: (aportlan) dont think this is the right memory layout
pub struct BoxUnswizzled<T, A> {
    pub ptr: SandboxPointer<T>,
    pub phantom_data: PhantomData<A>
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

pub(crate) fn unswizzle_nonnull<T>(nn: NonNull<T>) -> NonNullUnswizzled<T> {
    NonNullUnswizzled { pointer: unswizzle_ptr(nn.pointer as *mut T) }
}

pub(crate) fn swizzle_nonnull<T>(nn: &NonNullUnswizzled<T>, sandbox_ptr: usize) -> NonNull<T> {
    NonNull { pointer: swizzle_ptr(&nn.pointer, sandbox_ptr) }
}

pub(crate) fn unswizzle_raw_myvec<T, A: Allocator>(myvec: RawMyVec<T, A>) -> RawMyVecUnswizzled<T> {
    RawMyVecUnswizzled { 
        ptr: unswizzle_nonnull(myvec.ptr), 
        cap: myvec.cap as u32,
    }
}

pub(crate) fn swizzle_raw_myvec<T>(myvec: &RawMyVecUnswizzled<T>, sandbox_ptr: usize) -> RawMyVec<T> {
    RawMyVec { 
        ptr: swizzle_nonnull(&myvec.ptr, sandbox_ptr), 
        cap: myvec.cap as usize,
        alloc: Global
    }
}

pub(crate) fn unswizzle_myvec<T, A: Allocator>(myvec: MyVec<T, A>) -> MyVecUnswizzled<T> {
    MyVecUnswizzled { buf: unswizzle_raw_myvec(myvec.buf), len: myvec.len as u32 }
}

pub(crate) fn swizzle_myvec<T>(myvec: &MyVecUnswizzled<T>, sandbox_ptr: usize) -> MyVec<T> {
    MyVec { buf: swizzle_raw_myvec(&myvec.buf, sandbox_ptr), len: myvec.len as usize }
}