use std::{any::Any, fmt::Debug, io::Split};

use alohomora_sandbox::{alloc::{AllocateableInSandbox, SandboxAllocator}, copy::Copiable, swizzle::Swizzleable, unlock_sandbox};
use rocket::shield::Policy;
use serde::{Serialize, Deserialize};

use crate::{fold::new_fold, AlohomoraType, Foldable, SpecializeFoldable};
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;

#[derive(Debug, Clone)]
pub struct SplitSet {
    pub fold: u64,
    pub create: u64,
    pub alloc: u64,
    pub copy: u64,
    pub unswizzle: u64,
    pub invoke: u64,
}

impl SplitSet {
    pub fn sum(&self) -> u64 {
        self.fold + self.create + self.alloc + self.copy + self.unswizzle + self.invoke
    }
}

pub static mut SPLITS: Vec<SplitSet> = Vec::new();

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

    pub fn splits() -> Vec<SplitSet>{
        unsafe{ SPLITS.clone() }
    }

    pub unsafe fn split_info() {
        // get averages
        let fold_avg = SPLITS.iter().map(|split|{split.fold}).reduce(|a, b| a + b).unwrap() / (SPLITS.len() as u64);
        let create_avg = SPLITS.iter().map(|split|{split.create}).reduce(|a, b| a + b).unwrap() / (SPLITS.len() as u64);
        let alloc_avg = SPLITS.iter().map(|split|{split.alloc}).reduce(|a, b| a + b).unwrap() / (SPLITS.len() as u64);
        let copy_avg = SPLITS.iter().map(|split|{split.copy}).reduce(|a, b| a + b).unwrap() / (SPLITS.len() as u64);
        let unswizzle_avg = SPLITS.iter().map(|split|{split.unswizzle}).reduce(|a, b| a + b).unwrap() / (SPLITS.len() as u64);
        let invoke_avg = SPLITS.iter().map(|split|{split.invoke}).reduce(|a, b| a + b).unwrap() / (SPLITS.len() as u64);
        let total_avg = SPLITS.iter().map(|split|{split.sum()}).reduce(|a, b| a + b).unwrap() / (SPLITS.len() as u64);

        println!("");
        println!("----SPLIT INFO on {} runs----", SPLITS.len());
        println!("fold average: {:?}", fold_avg);
        println!("create average: {:?}", create_avg);
        println!("alloc average: {:?}", alloc_avg);
        println!("copy average: {:?}", copy_avg);
        println!("unswizzle average: {:?}", unswizzle_avg);
        println!("invoke average: {:?}", invoke_avg);
        println!("total average: {:?}", total_avg);
        println!("");

        println!("total average (no fold): {:?}", total_avg - fold_avg);
        println!("total average (no invoke): {:?}", total_avg - invoke_avg);
        println!("total average (no fold or invoke): {:?}", total_avg - fold_avg - invoke_avg);
    }

    fn percentile(vec: &mut Vec<u64>, perc: f64) -> u64 {
        vec.sort();
        let i = vec.len() / ((1.0 / perc) as usize);
        println!("have index {i} for len {:?} and perc {:?}", vec.len(), perc);
        vec[i]
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
        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // Remove boxes from args.
        let outer_boxed = new_fold(t).unwrap();
        let (t, p) = outer_boxed.consume();
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        let fold = end - start;
        println!("copy&execute - folding & consuming took {fold}");

        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // Create a new sandbox instance.
        let instance = SandboxInstance::new();
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        let create = end - start;
        println!("copy&execute - creating instance took {create}");

        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // Allocate space for the args in that sandbox instance.
        let mut inside = AllocateableInSandbox::allocate_in_sandbox(&t, &instance.alloc);
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        let alloc = end - start;
        println!("copy&execute - creating instance took {alloc}");

        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // Copy the args into the allocated space.
        unsafe { Copiable::copy(&mut inside, &t) };
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        let copy = end - start;
        println!("copy&execute - copying took {copy}");

        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // Unswizzle args for use in the sandbox.
        let final_arg = unsafe { Swizzleable::unswizzle(inside).into()};
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        let unswizzle = end - start;
        println!("copy&execute - unswizzling took {unswizzle}");

        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // Pass that to the function.
        let ret = S::invoke(final_arg, instance.sandbox_index);
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        let invoke = end - start;
        println!("copy&execute - invoking function took {invoke}");

        unsafe { SPLITS.push(SplitSet { fold, create, alloc, copy, unswizzle, invoke }); }

        BBox::new(ret, p)
    }

    /// Executes `S` on variable `t` assuming that `t` is already in sandboxed memory 
    /// (i.e. it has been allocated with this `SandboxInstance`'s `SandboxAllocator`).
    pub fn execute<'a, 'b, S, T, R, A>(self, t: T) -> BBox<::alohomora_sandbox::FinalSandboxOut<R>, AnyPolicy>
        where
                                         
            T: AlohomoraType<AnyPolicy, SandboxAllocator>,                      //     To ensure `T` is Alohomora type and using a sandbox allocator
            A: AllocateableInSandbox + Swizzleable,                             // <--
            T::Out: Into<<A as AllocateableInSandbox>::UsingSandboxAllocator>,  // <-| To ensure `T::Out` uses a sandbox allocator
            <A as AllocateableInSandbox>::UsingSandboxAllocator: Swizzleable,
            A::Unswizzled: From<<<A as AllocateableInSandbox>::UsingSandboxAllocator as Swizzleable>::Unswizzled>, // can't remember why we need this
            R: Serialize + Deserialize<'b>,
            S: AlohomoraSandbox<'a, 'b, A, R>,
    {
        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // 1. remove bboxes.
        let outer_boxed = fold(t).unwrap();
        let (t, p) = outer_boxed.consume();
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        println!("execute - folding took {:?}", end - start);

        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // 2. unswizzle data type.
        let sandbox_alloc: <A as AllocateableInSandbox>::UsingSandboxAllocator = t.into();
        let final_arg = unsafe { Swizzleable::unswizzle(sandbox_alloc).into() };
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        println!("execute - unswizzling took {:?}", end - start);

        let start = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        // 3. call functor.
        let ret = S::invoke(final_arg, self.sandbox_index);
        let end = mysql::chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        println!("execute - calling functor took {:?}", end - start);

        BBox::new(ret, p)
    }
}

impl Drop for SandboxInstance {
    fn drop(&mut self) {
        // Unlock sandbox mutex when this goes out of scope.
        unsafe{ unlock_sandbox(self.sandbox_index); }
    }
}