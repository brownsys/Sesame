use std::fmt::{Debug, Formatter, Result};
use std::sync::{Arc, Mutex};

use crate::policy::{Context, Policy};

pub struct BBox<T> {
    pub(crate) t: T,
    pub(crate) p: Vec<Arc<Mutex<dyn Policy>>>,
}
impl<T> BBox<T> {
    pub fn new(t: T, p: Vec<Arc<Mutex<dyn Policy>>>) -> Self {
        Self { t, p }
    }

    pub fn as_ref(&self) -> BBox<&T> {
        self.map(|t| t)
    }

    // Into that moves.
    pub fn into2<F>(self) -> BBox<F>
    where
        T: Into<F>,
    {
        self.into_map(|t| t.into())
    }

    // Unbox with policy checks.
    pub fn test_unbox(&self) -> &T {
        &self.t
    }
    pub fn unbox<U: 'static, D: 'static>(&self, _ctx: &Context<U, D>) -> &T {
        &self.t
    }
    pub fn into_unbox<U: 'static, D: 'static>(self, _ctx: &Context<U, D>) -> T {
        self.t
    }

    // Sandbox functions
    pub fn sandbox_execute<R, F: FnOnce(T) -> R>(self, lambda: F) -> BBox<R> {
        // Do we check policies?
        // Do we check that function is pure?
        // Do we execute in an actual sandbox?
        self.into_map(lambda)
    }
    pub fn into_sandbox_execute<'a, R, F: FnOnce(&'a T) -> R>(&'a self, lambda: F) -> BBox<R> {
        // Do we check policies?
        // Do we check that function is pure?
        // Do we execute in an actual sandbox?
        self.map(lambda)
    }

    // internal function to simplify creating BBoxes out of existing ones.
    pub(crate) fn into_map<R, F: FnOnce(T) -> R>(self, lambda: F) -> BBox<R> {
        BBox::new(lambda(self.t), self.p)
    }
    pub(crate) fn map<'a, R, F: FnOnce(&'a T) -> R>(&'a self, lambda: F) -> BBox<R> {
        BBox::new(lambda(&self.t), self.p.clone())
    }
}

// Debuggable but in boxed form.
impl<T> Debug for BBox<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str("<<Boxed Data>>")
    }
}

// BBox is clonable if what is inside is cloneable.
impl<T: Clone> Clone for BBox<T> {
    fn clone(&self) -> Self {
        self.map(|t| t.clone())
    }
}
