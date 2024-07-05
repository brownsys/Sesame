use std::{any::Any, fmt::Debug};

use alohomora_sandbox::{alloc::{AllocateableInSandbox, SandboxAllocator}, copy::Copiable, swizzle::Swizzleable, unlock_sandbox};
use rocket::shield::Policy;
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
    alloc: SandboxAllocator,
}

impl SandboxInstance {
    /// Create new sandbox instance. (in reality just get one from the pool)
    pub fn new() -> Self {
        let sandbox_index = unsafe{ ::alohomora_sandbox::get_lock_on_sandbox() };
        SandboxInstance { sandbox_index, alloc: SandboxAllocator::new(sandbox_index) }
    }

    /// An allocator to allocate into this instance's sandbox.
    pub fn alloc(&self) -> SandboxAllocator {
        self.alloc.clone()
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

    /// Executes `S` on variable `t` assuming that `t` is already in sandboxed memory 
    /// (i.e. it has been allocated with this `SandboxInstance`'s `SandboxAllocator`).
    pub fn execute<'a, 'b, S, T, R, P: crate::policy::Policy, A>(self, t: T) -> BBox<::alohomora_sandbox::FinalSandboxOut<R>, AnyPolicy>
        where
            A: AllocateableInSandbox + Swizzleable,
            T: AlohomoraType<P, SandboxAllocator>,
            T::Out: Into<<A as AllocateableInSandbox>::UsingSandboxAllocator>,
            <A as AllocateableInSandbox>::UsingSandboxAllocator: Swizzleable,
            A::Unswizzled: From<<<A as AllocateableInSandbox>::UsingSandboxAllocator as Swizzleable>::Unswizzled>,
            // <T::Out as Swizzleable>::Unswizzled: From<<<T::Out as AllocateableInSandbox>::UsingSandboxAllocator as Swizzleable>::Unswizzled>, // they shoudl really be the same but this is how im representing it
            // <T::Out as AllocateableInSandbox>::UsingSandboxAllocator: Swizzleable + Clone + Debug,
            R: Serialize + Deserialize<'b>,
            S: AlohomoraSandbox<'a, 'b, A, R>,
    {
        // 1. remove bboxes.
        println!("doing box business");
        let outer_boxed = fold(t).unwrap();
        let (t, p) = outer_boxed.consume();
        println!("done w box business");

        // 2. unswizzle data type.
        let sandbox_alloc: <A as AllocateableInSandbox>::UsingSandboxAllocator = t.into();
        let final_arg = unsafe { Swizzleable::unswizzle(sandbox_alloc).into() };
        println!("done deswizzling w inside");

        // 3. call functor.
        let ret = S::invoke(final_arg, self.sandbox_index);
        println!("done invoking");
        BBox::new(ret, p)
    }
}

impl Drop for SandboxInstance {
    fn drop(&mut self) {
        // Unlock sandbox mutex when this goes out of scope.
        unsafe{ unlock_sandbox(self.sandbox_index); }
    }
}