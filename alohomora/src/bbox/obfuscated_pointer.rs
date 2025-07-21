use std::boxed::Box;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Poll;

// Secret to XOR with.
const SECRET: usize = 2238711266;

pub struct ObPtr<T> {
    ptr: usize,
    _marker: PhantomData<T>,
}

impl<T> ObPtr<T> {
    // New obfuscated pointer puts data in a box, turns it to a raw pointer
    // and obfuscates the pointer.
    pub fn new(t: T) -> Self {
        let t = Box::new(t);
        let ptr: *mut T = Box::into_raw(t);
        let ptr: usize = ptr as usize;
        let ptr: usize = ptr ^ SECRET;
        Self {
            ptr: ptr,
            _marker: PhantomData,
        }
    }

    // No need to put in box, can deobfuscate directly.
    // Lifetime is same as ObPtr ref.
    pub fn get(&self) -> &T {
        unsafe { &*((self.ptr ^ SECRET) as *mut T) }
    }

    // Lifetime is same as ObPtr ref.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *((self.ptr ^ SECRET) as *mut T) }
    }

    // Consumes and moves the content out.
    // Deobfuscates, puts in a box, then moves out of the box.
    pub fn mov(mut self) -> T {
        let ptr = self.ptr;
        self.ptr = 0; // Zero it out to avoid double free.
                      // Convert the pointer back to a Box<T>, and give ownership of the value
        unsafe { *Box::from_raw((ptr ^ SECRET) as *mut T) }
    }
}

// Drop de-obfuscated the pointer, builds a Box, and drops it.
impl<T> Drop for ObPtr<T> {
    fn drop(&mut self) {
        if self.ptr != 0 {
            drop(unsafe { Box::from_raw((self.ptr ^ SECRET) as *mut T) });
        }
    }
}

// Cloneable if what's underneath is cloneable.
impl<T: Clone> Clone for ObPtr<T> {
    fn clone(&self) -> Self {
        ObPtr::new(self.get().clone())
    }
}

// This will be removed once we fix up ORM.
impl<T: PartialEq> PartialEq for ObPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}
impl<T: Debug> Debug for ObPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

// Can poll on futures inside ObPtr.
impl<'a, T: Future + Unpin> Future for ObPtr<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let inner_future: &mut T = unsafe { self.get_unchecked_mut().get_mut() };
        Pin::new(inner_future).poll(cx)
    }
}
