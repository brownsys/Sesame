use std::{alloc::{Allocator, Global}, convert::TryInto, fmt::Debug, marker::PhantomData};
use crate::{alloc::{AllocateableInSandbox, SandboxAllocator}, copy::{Copiable, Swizzleable, SwizzleableIdentity}, ptr::*, vec::{MyVec, NonNull, RawMyVec}};
use chrono::naive::NaiveDateTime;

impl<T: SwizzleableIdentity> Swizzleable for Vec<T, SandboxAllocator> {
    type Unswizzled = MyVecUnswizzled<T>;
    unsafe fn unswizzle(inside: Self) -> MyVecUnswizzled<T> {
        let ptr = (&inside as *const Vec<T, SandboxAllocator>) as *const crate::vec::MyVec<T, SandboxAllocator>;
        // let old_inside = (*ptr).clone(); 
        // ^^ special `MyVec` clone that only clones metadata & ptr not actual vec

        MyVecUnswizzled {
            buf: RawMyVecUnswizzled{
                ptr: NonNullUnswizzled{pointer: unswizzle_ptr((*ptr).buf.ptr.pointer as *mut T)},
                cap: (*ptr).buf.cap as u32,
            },
            len: (*ptr).len as u32,
        }
    }

    // unsafe fn swizzle(inside: Self::Unswizzled) -> Self 
    //     where Self: Sized {
        
    //     let v = MyVec {
    //         buf: crate::vec::RawMyVec { 
    //             ptr: crate::vec::NonNull{pointer: swizzle_ptr(&inside.buf.ptr.pointer, &mut inside as *mut Self::Unswizzled)}, 
    //             cap: inside.buf.cap as usize, 
    //             alloc: Global 
    //         },
    //         len: inside.len as usize,
    //     };
    // }
}

impl <T> Swizzleable for Vec<T> {
    type Unswizzled = MyVecUnswizzled<T>;
    unsafe fn unswizzle(_inside: Self) -> MyVecUnswizzled<T> {
        // shouldn't ever actually use this bc we should only use sandbox allocated vecs
        todo!();
    }
}

pub struct BoxUnswizzled<T, A> {
    pub ptr: SandboxPointer<T>,
    pub phantom_data: PhantomData<A>
}

/// New mega trait that handles copying into sandbox & unswizzling
pub trait Sandboxable {
    type InSandboxUnswizzled;

    /// Deeply move object `outside` into sandbox memory & recursively swizzle it.
    /// General approach for this takes two steps: 
    ///     1) recursively move everything this type points to into sandboxed memory
    ///     2) then swizzle this type's stack data (to be boxed and passed into sandbox)
    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled;

    /// Deeply copy `inside` out of sandbox memory.
    /// General approach is in the opposite order of `into_sandbox`:
    ///     1) swizzle out this type's stack data
    ///     2) then recursively move everything it points to out of the sandbox
    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, any_sandbox_ptr: usize) -> Self where Self: Sized {
        todo!()
    }
}

fn unswizzle_nonnull<T>(nn: NonNull<T>) -> NonNullUnswizzled<T> {
    NonNullUnswizzled { pointer: unswizzle_ptr(nn.pointer as *mut T) }
}

fn swizzle_nonnull<T>(nn: &NonNullUnswizzled<T>, sandbox_ptr: usize) -> NonNull<T> {
    NonNull { pointer: swizzle_ptr(&nn.pointer, sandbox_ptr) }
}

fn unswizzle_raw_myvec<T, A: Allocator>(myvec: RawMyVec<T, A>) -> RawMyVecUnswizzled<T> {
    RawMyVecUnswizzled { 
        ptr: unswizzle_nonnull(myvec.ptr), 
        cap: myvec.cap as u32,
    }
}

fn swizzle_raw_myvec<T>(myvec: &RawMyVecUnswizzled<T>, sandbox_ptr: usize) -> RawMyVec<T> {
    RawMyVec { 
        ptr: swizzle_nonnull(&myvec.ptr, sandbox_ptr), 
        cap: myvec.cap as usize,
        alloc: Global
    }
}

fn unswizzle_myvec<T, A: Allocator>(myvec: MyVec<T, A>) -> MyVecUnswizzled<T> {
    MyVecUnswizzled { buf: unswizzle_raw_myvec(myvec.buf), len: myvec.len as u32 }
}

fn swizzle_myvec<T>(myvec: &MyVecUnswizzled<T>, sandbox_ptr: usize) -> MyVec<T> {
    MyVec { buf: swizzle_raw_myvec(&myvec.buf, sandbox_ptr), len: myvec.len as usize }
}

impl<T: Sandboxable + Debug> Sandboxable for Vec<T> 
where T::InSandboxUnswizzled: Debug {
    type InSandboxUnswizzled = MyVecUnswizzled<T::InSandboxUnswizzled>;

    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled {
        // 1. move everything inside to sandbox
        let mut sandbox_vec = Vec::new_in(alloc.clone());
        outside.into_iter().map(|b|{
            Sandboxable::into_sandbox(b, alloc.clone())
        }).collect_into(&mut sandbox_vec);

        // 1b. convert to myvec so we can access private members
        let ptr: *const Vec<T::InSandboxUnswizzled, SandboxAllocator> = &sandbox_vec as *const Vec<T::InSandboxUnswizzled, SandboxAllocator>;
        let ptr = ptr as *mut MyVec<T::InSandboxUnswizzled, SandboxAllocator>;
        let a = unsafe { (*ptr).clone() };

        // 2. swizzle our metadata on the stack
        unswizzle_myvec(a)
    }

    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, sandbox_ptr: usize) -> Self where Self: Sized {
        // 2. swizzle our metadata on the stack
        let new_stack = swizzle_myvec(inside, sandbox_ptr);
        
        // 1b. convert back to real vec
        let p = Box::into_raw(Box::new(new_stack)) as *mut Vec<T::InSandboxUnswizzled>;
        let v = unsafe{ Box::leak(Box::from_raw(p)) };

        // 1. recursively bring all items out of the sandbox
        v.iter().map(|u| {
            Sandboxable::out_of_sandbox(u, sandbox_ptr)
        }).collect::<Vec<T>>()
    }
}

impl<T: Sandboxable> Sandboxable for Box<T> {
    type InSandboxUnswizzled = BoxUnswizzled<T::InSandboxUnswizzled, SandboxAllocator>; // TODO: is this the right memory layout

    fn into_sandbox(outside: Self, alloc: SandboxAllocator) -> Self::InSandboxUnswizzled {
        // 1. move boxed value into the sandbox portion of memory
        let new_val = Sandboxable::into_sandbox(*outside, alloc.clone());
        let b = Box::new_in(new_val, alloc);

        // 2. convert to a sandbox box w 32 bit ptr
        let ptr = Box::into_raw(b);
        let new_b = BoxUnswizzled { ptr: unswizzle_ptr(ptr), phantom_data: PhantomData::<SandboxAllocator> };
        new_b
    }
}

impl Sandboxable for usize {
    type InSandboxUnswizzled = u32;
    fn into_sandbox(outside: Self, _: SandboxAllocator) -> Self::InSandboxUnswizzled {
        outside.try_into().unwrap()
    }
}

impl Sandboxable for (usize, (), usize) {
    type InSandboxUnswizzled = (u32, (), u32);
    fn into_sandbox(outside: Self, _: SandboxAllocator) -> Self::InSandboxUnswizzled {
        (outside.0.try_into().unwrap(), (), outside.2.try_into().unwrap())
    }
    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, _: usize) -> Self where Self: Sized {
        (inside.0.try_into().unwrap(), (), inside.2.try_into().unwrap())
    }
}

macro_rules! derive_sandboxable_identity {
    ($t:ty) => {
        impl Sandboxable for $t {
            type InSandboxUnswizzled = $t;
            fn into_sandbox(outside: Self, _: SandboxAllocator) -> Self::InSandboxUnswizzled { outside }
            fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, _: usize) -> Self where Self: Sized { inside.clone() }
        }
    }
}

derive_sandboxable_identity!((NaiveDateTime, u64));
derive_sandboxable_identity!((u64, (), u64));
derive_sandboxable_identity!(u8);
derive_sandboxable_identity!(u32);


// TODO: (aportlan) dec macro for deriving that for identity types (those that dont change when swizzled)

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