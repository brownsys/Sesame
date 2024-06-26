use std::{collections::HashMap, hash::Hash, marker::PhantomData, os::raw::c_void};
use alohomora_derive::Swizzleable;
use once_cell::sync::Lazy;


static mut LOCAL_TO_SANDBOX: Lazy<HashMap<usize, usize>> = Lazy::new(||{
    HashMap::new()
});


// The sandbox pointer type. T represents what the pointer points to (for helpful type-checking)
#[derive(Debug, Clone, Copy)]
pub struct SandboxPointer<T> {
    pub ptr: u32,                   // actual 4 byte pointer
    _phantom: PhantomData<T>
}

impl<T> SandboxPointer<T> {
    pub fn new(ptr: u32) -> Self {
        SandboxPointer { ptr, _phantom: PhantomData::default() }
    }
}
// tells us which 

#[derive(Debug)]
pub struct Parent {
    pub cookouts_held: u32,
    pub hours_at_work: u32,
    pub height: f64,
    pub favorite_kid: *mut Baby,
}

#[derive(Debug, Clone)]
pub struct Baby {
    pub goos_gaad: u32,
    pub iq: u32,
    pub height: f64,
}

// convert a sandbox pointer to one that will work globally
pub fn swizzle_ptr<T, U>(ptr: &SandboxPointer<T>, known_ptr: *mut U) -> *mut T {
    let known_ptr = known_ptr as *mut c_void;
    let top32: u64 = 0xFFFFFFFF00000000;
    let bot32: u32 = 0xFFFFFFFF;
    let example_ptr: u64 = known_ptr as u64;
    let base: u64 = example_ptr & top32;
    let swizzled: u64 = (ptr.ptr as u64) + base;
    return swizzled as *mut T;
}

// convert global pointer to one that will work inside the sandbox
pub fn unswizzle_ptr<T>(ptr: *mut T) -> SandboxPointer<T> {
    let top32: u64 = 0xFFFFFFFF00000000;
    let bot32: u64 = 0xFFFFFFFF;
    let ptr = ptr as u64;
    let swizzled: u64 = ptr & bot32;
    return SandboxPointer::<T>::new(swizzled as u32);
}

// **************************** UNSWIZZLED VERSIONS ****************************

#[derive(Debug)]
pub struct Unswizzled<S, U> {
    _unswizzled: *mut U,
    pub data: S,
}

// Baby stays the same bc it doesn't contain pointers

// Grandparent is the same but it
//      a) replaces all pointers with consistent SandboxPointers (really u32s)

// use global dict instead of Unswizzled<U, T> type.

pub unsafe fn swizzle_grand(u: *mut GrandparentUnswizzled) -> *mut Grandparent{
    // find global ptr to kid in sandbox
    let kid_sandbox_ptr = swizzle_ptr(&(*u).favorite_kid, u);
    // bring them out of the sandbox
    let kid_app_ptr = swizzle_parent(kid_sandbox_ptr);

    // [!] if i have no pointer arguments, I can just return my own pointer in the sandbox
    
    // put the new item in a box in app memory.
    let b = Box::new(
            // for each field in struct...
        Grandparent {
            // if its not a pointer, copy it
            cookies_baked: (*u).cookies_baked, 
            pickleball_rank: (*u).pickleball_rank, 
            height: (*u).height, 
            // if it is a pointer, 
            //      swizzle THE POINTER to it, 
            //      swizzle THE DATA TYPE (using that pointer), and then 
            //      return THE RETURNED POINTER
            favorite_kid: kid_app_ptr 
        }
    );

    // store the address in the sandbox this should be copied back into.
    let ptr = Box::into_raw(b);
    LOCAL_TO_SANDBOX.insert(ptr as usize, u as usize);

    ptr
}

pub unsafe fn swizzle_parent(u: *mut ParentUnswizzled) -> *mut Parent {
    let b = Box::new(
        Parent { 
            cookouts_held: (*u).cookouts_held, 
            hours_at_work: (*u).hours_at_work, 
            height: (*u).height, 
            favorite_kid: swizzle_ptr(&(*u).favorite_kid, u as *mut std::ffi::c_void) 
        }
    );

    let ptr = Box::into_raw(b);
    let k = ptr as usize;
    LOCAL_TO_SANDBOX.insert(ptr as usize, u as usize);

    ptr
}

pub trait Swizzleable {
    type Unswizzled;
    unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled) -> *mut Self::Unswizzled;
    // unsafe fn swizzle(inside: *mut Self::Unswizzled, outside: *mut Self) -> *mut Self;
}

#[derive(Debug, Swizzleable)]
pub struct Grandparent {
    pub cookies_baked: u32,
    pub pickleball_rank: u32,
    pub height: f64,
    pub favorite_kid: *mut Parent,
}

// #[derive(Debug, Clone)]
// pub struct GrandparentUnswizzled {
//     pub cookies_baked: u32,
//     pub pickleball_rank: u32,
//     pub height: f64,
//     pub favorite_kid: SandboxPointer<<Parent as Swizzleable>::Unswizzled>,
// }

// impl Swizzleable for Grandparent {
//     type Unswizzled = GrandparentUnswizzled;
//     unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled) -> *mut Self::Unswizzled {
//         println!("got unswizzled addr {:?} for swizzled addr {:?}", inside, outside);

//         (*inside).cookies_baked = (*outside).cookies_baked;
//         (*inside).pickleball_rank = (*outside).pickleball_rank;
//         (*inside).height = (*outside).height;
//         (*inside).favorite_kid = unswizzle_ptr(Swizzleable::unswizzle((*outside).favorite_kid, swizzle_ptr(&(*inside).favorite_kid, inside)));
//         inside
//     }
// }



#[derive(Debug, Clone)]
pub struct ParentUnswizzled {
    pub cookouts_held: u32,
    pub hours_at_work: u32,
    pub height: f64,
    pub favorite_kid: SandboxPointer<<Baby as Swizzleable>::Unswizzled>,
}

impl Swizzleable for Parent {
    type Unswizzled = ParentUnswizzled;
    unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled) -> *mut Self::Unswizzled {
        println!("got unswizzled addr {:?} for swizzled addr {:?}", inside, outside);

        (*inside).cookouts_held = (*outside).cookouts_held;
        (*inside).hours_at_work = (*outside).hours_at_work;
        (*inside).height = (*outside).height;
        (*inside).favorite_kid = unswizzle_ptr(Swizzleable::unswizzle((*outside).favorite_kid, swizzle_ptr(&(*inside).favorite_kid, inside)));
        inside
    }
}

impl Swizzleable for Baby {
    type Unswizzled = Baby;
    unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled) -> *mut Self::Unswizzled {
        (*inside).goos_gaad = (*outside).goos_gaad;
        (*inside).iq = (*outside).iq;
        (*inside).height = (*outside).height;
        inside
    }
}