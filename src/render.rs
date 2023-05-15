use std::borrow::Cow;
use std::collections::HashMap;

use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::BBox;

pub trait InternalUnbox<T: Serialize> {
  fn internal_unbox(&self) -> &T;
}

impl InternalUnbox<i32> for i32 {
  fn internal_unbox(&self) -> &i32 {
    self
  }
}

impl InternalUnbox<String> for String {
  fn internal_unbox(&self) -> &String {
    self
  }
}

impl<T: Clone + Serialize> InternalUnbox<T> for BBox<T> {
  fn internal_unbox(&self) -> &T {
    self.internal_unbox()
  }
}

pub fn render_boxed<S, C, T>(name: S, context: &HashMap<&str, C>) -> Template
  where S: Into<Cow<'static, str>>, C: InternalUnbox<T>, T: Clone + Serialize {

  let mut unboxed_context: HashMap<&str, T> = HashMap::new();

  for (key, value) in context.iter() {
    unboxed_context.insert(key, value.internal_unbox().clone());
  }

  Template::render(name, unboxed_context)
}