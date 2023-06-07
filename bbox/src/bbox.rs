use std::fmt;
use core::fmt::Display;

use crate::policy::Policy;

pub struct BBox<T> {
  pub(crate) t: T,
  pub(crate) policies: Vec<Box<dyn Policy>>,
}

// BBox and containers of it are sandboxable.
pub trait Sandboxable<T> {
  type Out;  
  fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> Self::Out;
}
impl<T> Sandboxable<T> for BBox<T> {
  type Out = T;
  fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> T {
    lambda(self)
  }
}
impl<T, S: Sandboxable<T>> Sandboxable<T> for Vec<S> {
  type Out = Vec<S::Out>;
  fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> Self::Out {
    self.iter().map(|e| e.smap(&lambda)).collect()
  }
}
impl<T, S: Sandboxable<T>> Sandboxable<T> for &Vec<S> {
  type Out = Vec<S::Out>;
  fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> Self::Out {
    self.iter().map(|e| e.smap(&lambda)).collect()
  }
}


// Box functions.
impl<T> BBox<T> {
  // TODO(babman): We have not thought yet about how boxes get created initially,
  //               probably we need the policy here too.
  pub fn new(t: T) -> Self {
    Self { t: t, policies: vec![] }
  }
  
  // TODO(babman): new_with_policy should replace new.
  pub fn new_with_policy(t: T, policies: Vec<Box<dyn Policy>>) -> Self {
    Self { t, policies }
  }

  // Common operations that we are pulling into our library.
  // TODO(babmna): Can we get this to work with into/as etc in a more ergonomic way?
  pub fn into2<F>(
      &self
  ) -> BBox<F> where T: Into<F> + Clone {
    BBox::new(self.t.clone().into())
  }

  // Into that moves.
  pub fn m_into2<F>(self) -> BBox<F> where T: Into<F> {
    BBox::new(self.t.into())
  }
  // Converts &BBox<T> to BBox<&T>.
  pub fn as_ref(&self) -> BBox<&T> {
    BBox::new(&self.t)
  }

  // Unbox given a context (need more thinking)
  // TODO(babman): check policy here, make this take a context.
  pub fn unbox(&self, _ctx: &str) -> &T {
    &self.t
  }

  // Sandbox functions
  pub fn sandbox_execute<R, F: Fn(&T) -> R>(
      &self,
      lambda: F
  ) -> BBox<R> {
    BBox::new(lambda(&self.t))
  }
}

// String format.
impl<T: Display> BBox<T> {
  pub fn format(&self) -> BBox<String> {
    BBox::new(format!("{}", self.t))
  }
}

// Sandbox execute with a container of bboxes.
pub fn sandbox_execute<
    T: Clone,
    S: Sandboxable<T>,
    R,
    F: Fn(S::Out) -> R
>(
    s: S, lambda: F
) -> BBox<R> {
  let v = s.smap(|b| b.t.clone());
  BBox::new(lambda(v))
}

// Need to generalize this to many arguments.
pub fn sandbox_combine<
    T1: Clone,
    T2: Clone,
    S1: Sandboxable<T1>,
    S2: Sandboxable<T2>,
    R,
    F: Fn(S1::Out, S2::Out) -> R
>(
    s1: S1, s2: S2, lambda: F
) -> BBox<R> {
  let v1 = s1.smap(|b| b.t.clone());
  let v2 = s2.smap(|b| b.t.clone());
  BBox::new(lambda(v1, v2))
}

// Move BBox inside and outside a vec.
impl<T> From<BBox<Vec<T>>> for Vec<BBox<T>> {
  fn from(x: BBox<Vec<T>>) -> Vec<BBox<T>> {
    x.t.into_iter().map(|t| BBox::new(t)).collect()
  }
}
impl<T> From<Vec<BBox<T>>> for BBox<Vec<T>> {
  fn from(x: Vec<BBox<T>>) -> BBox<Vec<T>> {
    BBox::new(x.into_iter().map(|b| b.t).collect())
  }
}

// TODO(babman): These should be eventually removed.
impl<T> BBox<T> {
  // Usage of these should be pulled into our library.
  pub fn internal_new(t: T) -> Self {
    Self { t: t, policies: vec![] }
  }
  pub fn internal_unbox(&self) -> &T {
    &self.t
  }
}

// Debuggable but in boxed form.
impl<T> fmt::Debug for BBox<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("<<Boxed Data>>")
  }
}

// BBox is clonable if what is inside is cloneable.
impl<T: Clone> Clone for BBox<T> {
  fn clone(&self) -> Self {
    BBox::new(self.t.clone())
  }
}

// A type that contains either T or BBox<T>.
pub enum VBox<T> {
  Value(T),
  BBox(BBox<T>),
}

// VBox is clonable if T is clonable.
impl<T: Clone> Clone for VBox<T> {
  fn clone(&self) -> Self {
    match self {
      VBox::Value(value) => VBox::Value(value.clone()),
      VBox::BBox(bbox) => VBox::BBox(bbox.clone()),
    }
  }
}

// From for obvious types.
impl From<String> for VBox<String> {
  fn from(x: String) -> VBox<String> {
    VBox::Value(x)
  }
}
impl<T> From<BBox<T>> for VBox<T> {
  fn from(x: BBox<T>) -> VBox<T> {
    VBox::BBox(x)
  }
}


