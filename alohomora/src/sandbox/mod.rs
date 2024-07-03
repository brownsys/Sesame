use std::fmt::Debug;

use alohomora_sandbox::{alloc::{AllocateableInSandbox, SandboxAllocator}, copy::Copiable, swizzle::Swizzleable, unlock_sandbox};
use serde::{Serialize, Deserialize};

use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;

// Expose alohomora_sandbox API that controls the interface outside sandbox.
pub use alohomora_sandbox::{AlohomoraSandbox, FinalSandboxOut};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::AlohomoraSandbox;


// Main function for executing sandboxes over BBoxed data.
// pub fn execute_sandbox<'a, 'b, S, T, R>(t: T) -> BBox<::alohomora_sandbox::FinalSandboxOut<R>, AnyPolicy>
//     where
//         T: AlohomoraType,
//         T::Out: Clone + Swizzleable + AllocateableInSandbox, // TODO: might not need clone here
//         R: Serialize + Deserialize<'b>,
//         S: AlohomoraSandbox<'a, 'b, T::Out, R>,
// {
//     let outer_boxed = fold(t).unwrap();
//     let (t, p) = outer_boxed.consume();
//     BBox::new(S::invoke(t, 0), p)
// }

pub struct SandboxInstance {
    sandbox_index: usize,
    pub alloc: SandboxAllocator,
}

impl SandboxInstance {
    /// Create new sandbox instance. (in reality just get one from the pool)
    pub fn new() -> Self {
        let sandbox_index = unsafe{ ::alohomora_sandbox::get_lock_on_sandbox() };
        SandboxInstance { sandbox_index, alloc: SandboxAllocator::new(sandbox_index) }
    }

    /// Copies `t` into a sandbox and executes the specified function on it.
    pub fn copy_and_execute<'a, 'b, S, T, R>(t: T) -> BBox<::alohomora_sandbox::FinalSandboxOut<R>, AnyPolicy>
        where
            T: AlohomoraType,
            T::Out: AllocateableInSandbox + Copiable + Swizzleable + Debug,
            <T::Out as Swizzleable>::Unswizzled: 
                            From<<<T::Out as AllocateableInSandbox>::UsingSandboxAllocator as Swizzleable>::Unswizzled>, // they shoudl really just be the same but this is how im representing it
            <T::Out as AllocateableInSandbox>::UsingSandboxAllocator: Swizzleable + Clone + Debug,
            R: Serialize + Deserialize<'b>,
            S: AlohomoraSandbox<'a, 'b, T::Out, R>,
    {
        // Remove boxes from args.
        let outer_boxed = fold(t).unwrap();
        let (t, p) = outer_boxed.consume();

        // Create a new sandbox instance.
        let instance = SandboxInstance::new();

        // Allocate space for the args in that sandbox instance.
        let mut inside = AllocateableInSandbox::allocate_in_sandbox(&t, &instance.alloc);

        // Copy the args into the allocated space.
        unsafe { Copiable::copy(&mut inside, &t) };

        // Unswizzle args for use in the sandbox.
        let final_arg = unsafe { Swizzleable::unswizzle(inside).into()};

        // Pass that to the function.
        let ret = S::invoke(final_arg, instance.sandbox_index);
        BBox::new(ret, p)
    }

    // Executes `S` on variable `t` assuming that `t` is already in sandboxed memory 
    // (i.e. it has been allocated with this `SandboxInstance`'s `SandboxAllocator`).
    // pub fn execute<'a, 'b, S, T, R>(t: T) -> BBox<::alohomora_sandbox::FinalSandboxOut<R>, AnyPolicy>
    //     where
    //         T: AlohomoraType,
    //         T::Out: AllocateableInSandbox + Copiable + Swizzleable + Debug,
    //         <T::Out as Swizzleable>::Unswizzled: From<<<T::Out as AllocateableInSandbox>::UsingSandboxAllocator as Swizzleable>::Unswizzled>, // they shoudl really be the same but this is how im representing it
    //         <T::Out as AllocateableInSandbox>::UsingSandboxAllocator: Swizzleable + Clone + Debug,
    //         R: Serialize + Deserialize<'b>,
    //         S: AlohomoraSandbox<'a, 'b, T::Out, R>,
    // {
    //     println!("doing box business");
    //     let outer_boxed = fold(t).unwrap();
    //     let (t, p) = outer_boxed.consume();
    //     println!("done w box business");

    //     // 0. get lock on new sandbox
    //     let instance = SandboxInstance::new();
    //     println!("done making sandbox instance");

    //     // 1. allocate into the sandbox w out bboxes
    //     // should return a Vec<T, SandboxAllocator>
    //     let mut inside = AllocateableInSandbox::allocate_in_sandbox(&t, &instance.alloc);
    //     println!("done allocating w inside {:?}, {:p}", inside, &inside);

    //     // 2. move everything in there
    //     // should return a Vec<T> that is now in the sandbox
    //     println!("reminder--> t is {:?}", t);
    //     unsafe { Copiable::copy(&mut inside, &t) };
    //     println!("done copying w inside {:?}", inside);
    //     // println!("have vec {:?} at {:p}", inside, &inside);

    //     // 3. deswizzle all ptrs
    //     // Should return a deswizzled?
    //     let final_arg = unsafe { Swizzleable::unswizzle(inside).into()};
    //     println!("done deswizzling w inside");

    //     // 4. pass that into the function
    //     let ret = S::invoke(final_arg, instance.sandbox_index);
    //     println!("done invoking");
    //     BBox::new(ret, p)
    // }
}

impl Drop for SandboxInstance {
    fn drop(&mut self) {
        // Unlock sandbox mutex when this goes out of scope.
        unsafe{ unlock_sandbox(self.sandbox_index); }
    }
}