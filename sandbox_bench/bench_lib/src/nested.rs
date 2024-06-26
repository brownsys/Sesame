use std::{collections::HashMap, hash::Hash, marker::PhantomData, os::raw::c_void};
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
pub struct Grandparent {
    pub cookies_baked: u32,
    pub pickleball_rank: u32,
    pub height: f64,
    pub favorite_kid: *mut Parent,
}

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

pub fn swizzle_ptr<T, U>(ptr: &SandboxPointer<T>, known_ptr: *mut U) -> *mut T {
    let known_ptr = known_ptr as *mut c_void;
    let top32: u64 = 0xFFFFFFFF00000000;
    let bot32: u32 = 0xFFFFFFFF;
    let example_ptr: u64 = known_ptr as u64;
    let base: u64 = example_ptr & top32;
    let swizzled: u64 = (ptr.ptr as u64) + base;
    return swizzled as *mut T;
}

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

#[derive(Debug, Clone)]
pub struct GrandparentUnswizzled {
    pub cookies_baked: u32,
    pub pickleball_rank: u32,
    pub height: f64,
    pub favorite_kid: SandboxPointer<ParentUnswizzled>,
}

#[derive(Debug, Clone)]
pub struct ParentUnswizzled {
    pub cookouts_held: u32,
    pub hours_at_work: u32,
    pub height: f64,
    pub favorite_kid: SandboxPointer<Baby>,
}

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

pub unsafe fn unswizzle_grand(s: *mut Grandparent) -> *mut GrandparentUnswizzled {
    let unswizzled = *LOCAL_TO_SANDBOX.get(&(s as usize)).unwrap() as *mut GrandparentUnswizzled;
    println!("got unswizzled addr {:?} for swizzled addr {:?}", unswizzled, s);

    (*unswizzled).cookies_baked = (*s).cookies_baked;
    (*unswizzled).pickleball_rank = (*s).pickleball_rank;
    (*unswizzled).height = (*s).height;
    (*unswizzled).favorite_kid = unswizzle_ptr(unswizzle_parent((*s).favorite_kid));
    unswizzled
}

pub unsafe fn unswizzle_parent(s: *mut Parent) -> *mut ParentUnswizzled {

    let unswizzled = *LOCAL_TO_SANDBOX.get(&(s as usize)).unwrap() as *mut ParentUnswizzled;
    println!("got unswizzled PARENT addr {:?} for swizzled addr {:?}", unswizzled, s);

    (*unswizzled).cookouts_held = (*s).cookouts_held;
    (*unswizzled).hours_at_work = (*s).hours_at_work;
    (*unswizzled).height = (*s).height;
    (*unswizzled).favorite_kid = unswizzle_ptr((*s).favorite_kid);
    unswizzled
}






impl UnswizzledData for ParentUnswizzled {
    fn swizzle(&self) {
        
    }
}


pub trait UnswizzledData {
    // Swizzle whole data type out of sandboxed memory.
    fn swizzle(&self);
}

pub trait SwizzledData {
    // Swizzle whole data type back into sandboxed memory.
    fn unswizzle(&self);
}