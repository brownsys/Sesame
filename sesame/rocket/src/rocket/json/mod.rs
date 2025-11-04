use std::ops::{Deref, DerefMut};

mod json;
mod request;
mod response;

pub use json::*;
pub use request::*;
pub use response::*;

// Can use this as an argument or return this from route functions.
pub struct PConJson<T>(pub T);
impl<T> PConJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
impl<T> Deref for PConJson<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> DerefMut for PConJson<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
