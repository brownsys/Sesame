pub mod render;

extern crate rocket;

use rocket::form::{self, FromFormField, DataField, ValueField};
use std::fmt;
use serde::{Serialize, Serializer};

pub struct BBox<T: Clone + Serialize> {
  pub t: T,
}

impl<T: Clone + Serialize> fmt::Debug for BBox<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("<<Boxed Data>>")
  }
}

impl<T: Clone + Serialize> Serialize for BBox<T> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str("<<Boxed Data>>")
  }
}

impl<T: Clone + Serialize> BBox<T> {
  // TODO(babman): We have not thought yet about how boxes get created initially,
  //               probably we need the policy here too.
  pub fn new(t: T) -> Self {
    Self { t }
  }

  // Common operations that we are pulling into our library.
  // TODO(babmna): Can we get this to work with into/as etc in a more ergonomic way?
  pub fn into2<F: Clone + Serialize>(
      &self
  ) -> BBox<F> where T: Into<F> {
    BBox::new(self.t.clone().into())
  }
  
  // Usage of these should be pulled into our library.
  pub fn internal_new(t: T) -> Self {
    Self { t }
  }
  pub fn internal_unbox(&self) -> &T {
    &self.t
  }

  // Unbox given a context (need more thinking)
  pub fn unbox(&self, _ctx: &str) -> &T {
    &self.t
  }

  // Sandbox functions
  pub fn sandbox_execute<
      R: Clone + Serialize,
      F: Fn(&T) -> R
  >(
      &self,
      lambda: F
  ) -> BBox<R> {
    BBox::new(lambda(&self.t))
  }

  // Need to generalize this to many arguments.
  pub fn sandbox_combine<
      U: Clone + Serialize,
      V: Clone + Serialize,
      R: Clone + Serialize,
      F: Fn(&U, &V) -> R
  >(
      bbox1: BBox<U>,
      bbox2: BBox<V>, 
      lambda: F
  ) -> BBox<R> {
    BBox::new(lambda(bbox1.internal_unbox(), bbox2.internal_unbox()))
  }
}

#[rocket::async_trait]
impl<'r, T> FromFormField<'r> for BBox<T> where T: Send + Clone + Serialize {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        todo!("parse from a value or use default impl")
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        todo!("parse from a value or use default impl")
    }
}
