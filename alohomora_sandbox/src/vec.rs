use std::{alloc::{self, Layout}, fmt::Debug, marker::PhantomData, mem, ops::{Deref, DerefMut}, ptr};
use crate::{ptr::*, swizzle::Swizzleable};
use ::alohomora_derive::Swizzleable;

#[derive(Debug, Copy)]
pub struct NonNull<T: Sized> {
    pub pointer: *const T,
}

impl<T> Clone for NonNull<T> {
    fn clone(&self) -> Self {
        NonNull { pointer: self.pointer.clone() }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NonNullUnswizzled<T> {
    pub pointer: SandboxPointer<T>,
}

impl<T: Sized> NonNull<T> {
    pub const fn as_ptr(self) -> *mut T {
        self.pointer as *mut T
    }
    pub const fn dangling() -> Self {
        // SAFETY: mem::align_of() returns a non-zero usize which is then casted
        // to a *mut T. Therefore, `ptr` is not null and the conditions for
        // calling new_unchecked() are respected.
        unsafe {
            let ptr = mem::align_of::<T>() as *mut T;
            NonNull::new_unchecked(ptr)
        }
    }
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        unsafe {
            NonNull { pointer: ptr as _ }
        }
    }
    pub const fn new(ptr: *mut T) -> Option<Self> {
        Some(unsafe { Self::new_unchecked(ptr) })
    }
}
#[derive(Debug)]
pub struct RawMyVecUnswizzled<T> {
    pub ptr: NonNullUnswizzled<T>,
    pub cap: u32,
}
impl<T: Debug> Swizzleable for RawMyVec<T> {
    type Unswizzled = RawMyVecUnswizzled<T>;
    unsafe fn unswizzle(
        outside: *mut Self,
        inside: *mut Self::Unswizzled,
        old_inside: &Self,
    ) -> *mut Self::Unswizzled {
        // we need to save the old state of the inside to prevent it from being override with a new memory layout
        println!("a outside is {:?} and inside is {:?}", (*outside).ptr.pointer, old_inside);
        std::ptr::copy((*outside).ptr.pointer, old_inside.ptr.pointer as *mut T, (*outside).cap);
        println!("b");
        let inside = inside as *mut Self::Unswizzled;
        println!("c");
        (*inside).cap = (*outside).cap as u32;  // everything else gets copied from outside to move data
        println!("d");
        (*inside).ptr.pointer = unswizzle_ptr(old_inside.ptr.clone().as_ptr()); // ptrs get copied from old_inside to preserve data structure
        println!("e");
        inside as *mut Self::Unswizzled
    }
}

#[derive(Debug)]
pub struct RawMyVec<T> {
    pub ptr: NonNull<T>,
    pub cap: usize,
}

impl<T> Clone for RawMyVec<T> {
    fn clone(&self) -> Self {
        RawMyVec { ptr: self.ptr.clone(), cap: self.cap.clone() }
    }
}

#[derive(Debug)]
pub struct MyVec<T> {
    pub buf: RawMyVec<T>,
    pub len: usize,
}

impl<T> Clone for MyVec<T> {
    fn clone(&self) -> Self {
        MyVec { buf: self.buf.clone(), len: self.len.clone() }
    }
}

#[derive(Debug)]
pub struct MyVecUnswizzled<T> {
    pub buf: RawMyVecUnswizzled<T>,
    pub len: u32,
}

impl<T: Debug> Swizzleable for MyVec<T> {
    type Unswizzled = MyVecUnswizzled<T>;
    unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled, old_inside: &Self) -> *mut Self::Unswizzled {
        println!("f");
        let inside = inside as *mut Self::Unswizzled;
        println!("g");
        Swizzleable::unswizzle(&mut (*outside).buf as *mut RawMyVec<T>, &mut (*inside).buf as *mut RawMyVecUnswizzled<T>, &old_inside.buf);
        println!("h");
        (*inside).len = (*outside).len as u32;
        println!("i");
        inside
    }
}

impl<T> From<Vec<T>> for MyVec<T> {
    fn from(value: Vec<T>) -> Self {
        let b = Box::new(value);
        let ptr = Box::into_raw(b);
        let ptr = ptr as *mut MyVec<T>;
        let b = unsafe { Box::from_raw(ptr) };
        *b
    }
}

impl<T> From<MyVec<T>> for Vec<T> {
    fn from(value: MyVec<T>) -> Self {
        let b = Box::new(value);
        let ptr = Box::into_raw(b);
        let ptr = ptr as *mut Vec<T>;
        let b = unsafe { Box::from_raw(ptr) };
        *b
    }
}

impl<T: Clone + Debug> Swizzleable for Vec<T> {
    type Unswizzled = Vec<T>;
    unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled, old_inside: &Self) -> *mut Self::Unswizzled {
        let old_inside_ptr = inside as *mut MyVec<T>;
        let old_inside = (*old_inside_ptr).clone();
        Swizzleable::unswizzle(outside as *mut MyVec<T>, inside as *mut MyVecUnswizzled<T>, &old_inside) as *mut Vec<T>
    }
}

unsafe impl<T: Send> Send for RawMyVec<T> {}
unsafe impl<T: Sync> Sync for RawMyVec<T> {}

impl<T> RawMyVec<T> {
    fn new() -> Self {
        // !0 is usize::MAX. This branch should be stripped at compile time.
        let cap = if mem::size_of::<T>() == 0 { !0 } else { 0 };

        // `NonNull::dangling()` doubles as "unallocated" and "zero-sized allocation"
        RawMyVec {
            ptr: NonNull::dangling(),
            cap,
        }
    }

    fn grow(&mut self) {
        // since we set the capacity to usize::MAX when T has size 0,
        // getting to here necessarily means the MyVec is overfull.
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            // This can't overflow because we ensure self.cap <= isize::MAX.
            let new_cap = 2 * self.cap;

            // `Layout::array` checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // so the `unwrap` should never fail.
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.pointer as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }
}

impl<T> Drop for RawMyVec<T> {
    fn drop(&mut self) {
        let elem_size = mem::size_of::<T>();

        if self.cap != 0 && elem_size != 0 {
            // unsafe {
            //     alloc::dealloc(
            //         self.ptr.as_ptr() as *mut u8,
            //         Layout::array::<T>(self.cap).unwrap(),
            //     );
            // }
        }
    }
}

impl<T> MyVec<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.pointer as *mut T
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new() -> Self {
        MyVec {
            buf: RawMyVec::new(),
            len: 0,
        }
    }
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }

        // Can't overflow, we'll OOM first.
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), elem);
        }

        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        self.len -= 1;

        unsafe {
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
            result
        }
    }

    pub fn drain(&mut self) -> Drain<T> {
        let iter = unsafe { RawValIter::new(&self) };

        // this is a mem::forget safety thing. If Drain is forgotten, we just
        // leak the whole MyVec's contents. Also we need to do this *eventually*
        // anyway, so why not do it now?
        self.len = 0;

        Drain {
            iter,
            vec: PhantomData,
        }
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
        // deallocation is handled by RawMyVec
    }
}

impl<T> Deref for MyVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for MyVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<T> IntoIterator for MyVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        let (iter, buf) = unsafe {
            (RawValIter::new(&self), ptr::read(&self.buf))
        };

        mem::forget(self);

        IntoIter {
            iter,
            _buf: buf,
        }
    }
}

struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.start = (self.start as usize + 1) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    let old_ptr = self.start;
                    self.start = self.start.offset(1);
                    Some(ptr::read(old_ptr))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len = (self.end as usize - self.start as usize)
                  / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.end = (self.end as usize - 1) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    self.end = self.end.offset(-1);
                    Some(ptr::read(self.end))
                }
            }
        }
    }
}

pub struct IntoIter<T> {
    _buf: RawMyVec<T>, // we don't actually care about this. Just need it to live.
    iter: RawValIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

pub struct Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut MyVec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        // pre-drain the iter
        for _ in &mut *self {}
    }
}