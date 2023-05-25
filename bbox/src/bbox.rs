use std::fmt;

pub struct BBox<T> {
  pub(crate) t: T,
}

impl<T> BBox<T> {
  // TODO(babman): We have not thought yet about how boxes get created initially,
  //               probably we need the policy here too.
  pub fn new(t: T) -> Self {
    Self { t }
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

  // Unbox given a context (need more thinking)
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

  // Need to generalize this to many arguments.
  pub fn sandbox_combine<U, V, R, F: Fn(&U, &V) -> R>(
      bbox1: BBox<U>,
      bbox2: BBox<V>, lambda: F
  ) -> BBox<R> {
    BBox::new(lambda(bbox1.internal_unbox(), bbox2.internal_unbox()))
  }
}

// TODO(babman): These should be eventually removed.
impl<T> BBox<T> { 
  // Usage of these should be pulled into our library.
  pub fn internal_new(t: T) -> Self {
    Self { t }
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
