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

// Adapted from: https://api.rocket.rs/v0.5-rc/src/rocket_dyn_templates/lib.rs.html#522-557

#[macro_export]
macro_rules! context {
    ($($key:ident $(: $value:expr)?),*$(,)?) => {{
        use $crate::render::InternalUnbox;
        use serde::ser::{Serialize, Serializer, SerializeMap};
        use ::std::fmt::{Debug, Formatter};
        use ::std::result::Result;

        #[allow(non_camel_case_types)]
        struct ContextMacroCtxObject<$($key: Serialize),*> {
            $($key: $key),*
        }

        #[allow(non_camel_case_types)]
        impl<$($key: Serialize),*> Serialize for ContextMacroCtxObject<$($key),*> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                let mut map = serializer.serialize_map(None)?;
                $(map.serialize_entry(stringify!($key), &self.$key)?;)*
                map.end()
            }
        }

        #[allow(non_camel_case_types)]
        impl<$($key: Debug + Serialize),*> Debug for ContextMacroCtxObject<$($key),*> {
            fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct("context!")
                    $(.field(stringify!($key), &self.$key))*
                    .finish()
            }
        }

        ContextMacroCtxObject {
            $($key $(: InternalUnbox::internal_unbox($value))?),*
        }
    }};
}