use alohomora_derive::{AlohomoraType, Swizzleable};
use once_cell::sync::Lazy;
use alohomora_sandbox::*;
use alohomora_sandbox::{ptr::*, alloc::*};


// static mut LOCAL_TO_SANDBOX: Lazy<HashMap<usize, usize>> = Lazy::new(||{
//     HashMap::new()
// });

#[derive(Debug, Clone, Swizzleable)]
pub struct Parent {
    pub cookouts_held: u32,
    pub hours_at_work: u32,
    pub height: f64,
    pub favorite_kid: *mut Baby,
}

#[derive(Debug, Clone, Swizzleable)]
pub struct Baby {
    pub goos_gaad: u32,
    pub iq: u32,
    pub height: f64,
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

// pub unsafe fn swizzle_grand(u: *mut GrandparentUnswizzled) -> *mut Grandparent{
//     // find global ptr to kid in sandbox
//     let kid_sandbox_ptr = swizzle_ptr(&(*u).favorite_kid, u);
//     // bring them out of the sandbox
//     let kid_app_ptr = swizzle_parent(kid_sandbox_ptr);

//     // [!] if i have no pointer arguments, I can just return my own pointer in the sandbox
    
//     // put the new item in a box in app memory.
//     let b = Box::new(
//             // for each field in struct...
//         Grandparent {
//             // if its not a pointer, copy it
//             cookies_baked: (*u).cookies_baked, 
//             pickleball_rank: (*u).pickleball_rank, 
//             height: (*u).height, 
//             // if it is a pointer, 
//             //      swizzle THE POINTER to it, 
//             //      swizzle THE DATA TYPE (using that pointer), and then 
//             //      return THE RETURNED POINTER
//             favorite_kid: kid_app_ptr 
//         }
//     );

//     // store the address in the sandbox this should be copied back into.
//     let ptr = Box::into_raw(b);
//     LOCAL_TO_SANDBOX.insert(ptr as usize, u as usize);

//     ptr
// }

// pub unsafe fn swizzle_parent(u: *mut ParentUnswizzled) -> *mut Parent {
//     let b = Box::new(
//         Parent { 
//             cookouts_held: (*u).cookouts_held, 
//             hours_at_work: (*u).hours_at_work, 
//             height: (*u).height, 
//             favorite_kid: swizzle_ptr(&(*u).favorite_kid, u as *mut std::ffi::c_void) 
//         }
//     );

//     let ptr = Box::into_raw(b);
//     let k = ptr as usize;
//     LOCAL_TO_SANDBOX.insert(ptr as usize, u as usize);

//     ptr
// }

// impl AllocateableInSandbox for Parent {
//     unsafe fn allocate_in_sandbox(info: *mut Self, alloc: SandboxAllocator) -> *mut Self {
//         let mut b = Box::new_in((*info).clone(), alloc.clone());
//         (*b).favorite_kid = AllocateableInSandbox::allocate_in_sandbox((*b).favorite_kid, alloc);
//         Box::into_raw(b)
//     }
// }

// impl AllocateableInSandbox for Baby {
//     unsafe fn allocate_in_sandbox(info: *mut Self, alloc: SandboxAllocator) -> *mut Self {
//         // let old_box = Box::from_raw(info);
//         let b = Box::new_in((*info).clone(), alloc);
//         Box::into_raw(b)
//     }
// }
// #![feature(associated_type_bounds)]
// pub trait IntoSwizzleable {
//     unsafe fn to_swizzleable(non_swizzleable: *mut Self) -> Box<dyn Swizzleable>;
// }

// impl<S> IntoSwizzleable for S
//     where S: Swizzleable {
//     unsafe fn to_swizzleable<T: Swizzleable>(non_swizzleable: *mut Self) -> *mut T { 
//         non_swizzleable as *mut T
//     }
// }

// impl<V> ToSwizzleable<MyVec<V>> for Vec<V> {
//     unsafe fn to_swizzleable(non_swizzleable: *mut Self) -> *mut MyVec<V> {
//         non_swizzleable as *mut MyVec<V>
//     }
// }

#[derive(Debug, Clone, Swizzleable)]
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



// #[derive(Debug, Clone)]
// pub struct ParentUnswizzled {
//     pub cookouts_held: u32,
//     pub hours_at_work: u32,
//     pub height: f64,
//     pub favorite_kid: SandboxPointer<<Baby as Swizzleable>::Unswizzled>,
// }

// impl Swizzleable for Parent {
//     type Unswizzled = ParentUnswizzled;
//     unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled) -> *mut Self::Unswizzled {
//         println!("got unswizzled addr {:?} for swizzled addr {:?}", inside, outside);

//         (*inside).cookouts_held = (*outside).cookouts_held;
//         (*inside).hours_at_work = (*outside).hours_at_work;
//         (*inside).height = (*outside).height;
//         (*inside).favorite_kid = unswizzle_ptr(Swizzleable::unswizzle((*outside).favorite_kid, swizzle_ptr(&(*inside).favorite_kid, inside)));
//         inside
//     }
// }

// impl Swizzleable for Baby {
//     type Unswizzled = Baby;
//     unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled) -> *mut Self::Unswizzled {
//         (*inside).goos_gaad = (*outside).goos_gaad;
//         (*inside).iq = (*outside).iq;
//         (*inside).height = (*outside).height;
//         inside
//     }
// }