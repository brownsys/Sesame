extern crate figment;
extern crate erased_serde;

use std::borrow::Cow;
use std::collections::{HashMap, BTreeMap};

use dynfmt::{Format, SimpleCurlyFormat};
use erased_serde::Serialize;
use figment::Error as FError;
use figment::value::Value as FValue;
use rocket::response::Redirect;
use rocket_dyn_templates::Template;

// Our BBox struct.
use crate::{BBox, VBox};

// A BBox with type T erased, a primitive value, or a collection of mixed-type
// values.
pub enum Renderable<'a> {
  //Cloneable(BBox<Box<&'a dyn Clone + Serialize>>),
  BBox(BBox<&'a dyn Serialize>),
  Serialize(&'a dyn Serialize),
  Dict(BTreeMap<String, Renderable<'a>>),
  Array(Vec<Renderable<'a>>),
}

impl<'a> Renderable<'a> {
  pub(crate) fn transform(&self) -> Result<FValue, figment::Error> {
    match self {
      Renderable::BBox(bbox) => FValue::serialize(bbox.t),
      Renderable::Serialize(obj) => FValue::serialize(obj),
      Renderable::Dict(map) => {
        let mut tmap: BTreeMap<String, FValue> = BTreeMap::new();
        for (k, v) in map {
          let v = v.transform()?;
          tmap.insert(k.clone(), v);
        }
        Ok(FValue::from(tmap))
      },
      Renderable::Array(vec) => {
        let mut tvec: Vec<FValue> = Vec::new();
        for v in vec {
          let v = v.transform()?;
          tvec.push(v);
        }
        Ok(FValue::from(tvec))
      },
    }
  }

  pub(crate) fn try_unbox(&self) -> Result<&'a dyn Serialize, &'static str> {
    match self {
      Renderable::BBox(bbox) => Ok(bbox.t),
      Renderable::Serialize(obj) => Ok(*obj),
      Renderable::Dict(_) => Err("unsupported operation"),
      Renderable::Array(_) => Err("unsupported operation"),
    }
  }
}

// Anything that implements this trait can be rendered by our render wrapper.
// The `bbox_derive` lib provides macros to derive this for structs that consist
// of other BBoxRender fields.
pub trait BBoxRender {
  fn render<'a>(&'a self) -> Renderable<'a>;
}

// Auto implement BBoxRender for unboxed primitive types.
macro_rules! render_serialize_impl {
  ($T: ty) => (
    impl BBoxRender for $T {
      fn render<'a>(&'a self) -> Renderable<'a> {
        Renderable::Serialize(self)
      }
    }
  );
}
render_serialize_impl!(String);
render_serialize_impl!(u64);
render_serialize_impl!(i64);
render_serialize_impl!(u8);
render_serialize_impl!(i8);

// Auto implement BBoxRender for BBox.
impl<T: Serialize> BBoxRender for BBox<T> {
  fn render<'a>(&'a self) -> Renderable<'a> {
    Renderable::BBox(BBox::new(&self.t))
  }
}

// Auto implement BBoxRender for VBox.
impl<T: Serialize> BBoxRender for VBox<T> {
  fn render<'a>(&'a self) -> Renderable<'a> {
    match self {
      VBox::Value(value) => Renderable::Serialize(value),
      VBox::BBox(bbox) => Renderable::BBox(BBox::new(&bbox.t))
    }
  }
}

// Auto implement BBoxRender for Vec.
impl<T: BBoxRender> BBoxRender for Vec<T> {
  fn render<'a>(&'a self) -> Renderable<'a> {
    Renderable::Array(self.iter().map(|v| v.render()).collect())
  }
}

// Auto implement BBoxRender for HashMap.
impl<T: BBoxRender> BBoxRender for HashMap<&str, T> {
  fn render<'a>(&'a self) -> Renderable<'a> {
    let mut map = BTreeMap::new();
    for (key, val) in self.iter() {
      map.insert((*key).into(), val.render());
    }
    Renderable::Dict(map)
  }
}

// Our render wrapper takes in some BBoxRender type, transforms it to a figment
// Value compatible with Rocket, and then calls Rocket's render.
pub fn render<S: Into<Cow<'static, str>>, T: BBoxRender>(name: S, context: &T)
    -> Result<Template, FError> {
  // First turn context into a figment::value::Value.
  let transformed: FValue = context.render().transform()?;

  // Now render.
  Ok(Template::render(name, transformed))
}

// Our redirect wrapper operates similar to Rocket redirect, but takes in bbox
// parameters.
pub fn redirect(name: &str, params: Vec<&dyn BBoxRender>) -> Redirect {
  let formatted_params: Vec<&dyn Serialize> = params.iter().map(
    |x| x.render().try_unbox().unwrap().clone()).collect();
  let formatted_str: Cow<str> = SimpleCurlyFormat.format(name, formatted_params).unwrap();
  Redirect::to(Into::<String>::into(formatted_str))
}
