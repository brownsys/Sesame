use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

use alohomora_derive::Swizzleable;
use alohomora_sandbox::ptr::{unswizzle_ptr, swizzle_ptr, SandboxPointer};
use alohomora_sandbox::*;
use alohomora_sandbox::swizzle::Swizzleable;

// type SandboxPointer = u32;

// #[derive(Debug)]
// pub struct RawMyVecUnswizzled {
//     pub ptr: SandboxPointer,
//     pub cap: u32, // typically `usize`, but that's 4 bytes in sandbox and 8 outside
// }

// #[derive(Debug)]
// pub struct MyVecUnswizzled {
//     pub buf: RawMyVecUnswizzled,
//     pub len: u32,
// }

// pub unsafe fn swizzle<T>(unswizzled: *mut MyVecUnswizzled) -> Unswizzled<MyVec<T>, MyVecUnswizzled> {
//     let d = MyVec::<T> {
//         buf: RawMyVec { 
//             ptr: NonNull::new(::alohomora_sandbox::swizzle_ptr((*unswizzled).buf.ptr, unswizzled as *mut std::ffi::c_void)).unwrap(), 
//             cap: (*unswizzled).buf.cap as usize
//         },
//         len: (*unswizzled).len as usize,
//     };

//     Unswizzled {
//         _unswizzled: unswizzled,
//         data: d,
//     }
// }
// #[derive(Debug)]
// pub struct Unswizzled<S, U> {
//     _unswizzled: *mut U,
//     pub data: S,
// }

// impl<T> Unswizzled<MyVec<T>, MyVecUnswizzled> {
//     pub unsafe fn unswizzle(&self) {
//         (*self._unswizzled).len = self.data.len as u32;
//         (*self._unswizzled).buf.cap = self.data.buf.cap as u32;
//         (*self._unswizzled).buf.ptr = ::alohomora_sandbox::unswizzle_ptr(self.data.buf.ptr.as_ptr());
//     }
// }
