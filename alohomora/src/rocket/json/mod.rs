use std::ops::{Deref, DerefMut};

mod json;
mod request;
mod response;

pub use json::*;
pub use request::*;
pub use response::*;

// Can use this as an argument or return this from route functions.
pub struct BBoxJson<T>(pub T);
impl<T> BBoxJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
impl<T> Deref for BBoxJson<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> DerefMut for BBoxJson<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
