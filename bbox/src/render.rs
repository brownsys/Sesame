extern crate figment;
extern crate erased_serde;

use std::borrow::Cow;
use std::collections::BTreeMap;
use erased_serde::Serialize;

use rocket_dyn_templates::Template;
use figment::value::Value as FValue;
use figment::Error as FError;

// Our BBox struct.
use crate::BBox;

// A BBox with type T erased, a primitive value, or a collection of mixed-type
// values.
pub enum ValueOrBBox<'a> {
  //Cloneable(BBox<Box<&'a dyn Clone + Serialize>>),
  BBox(BBox<&'a dyn Serialize>),
  Serialize(&'a dyn Serialize),
  Dict(BTreeMap<String, ValueOrBBox<'a>>),
  Array(Vec<ValueOrBBox<'a>>),
}

impl<'a> ValueOrBBox<'a> {
  pub(crate) fn transform(&self) -> Result<FValue, figment::Error> {
    match self {
      ValueOrBBox::BBox(bbox) => FValue::serialize(bbox.safe_unbox()),
      ValueOrBBox::Serialize(obj) => FValue::serialize(obj),
      ValueOrBBox::Dict(map) => {
        let mut tmap: BTreeMap<String, FValue> = BTreeMap::new();
        for (k, v) in map {
          let v = v.transform()?;
          tmap.insert(k.clone(), v);
        }
        Ok(FValue::from(tmap))
      },
      ValueOrBBox::Array(vec) => {
        let mut tvec: Vec<FValue> = Vec::new();
        for v in vec {
          let v = v.transform()?;
          tvec.push(v);
        }
        Ok(FValue::from(tvec))
      },
    }
  }
}

// Anything that implements this trait can be rendered by our render wrapper.
// The `bbox_derive` lib provides macros to derive this for structs that consist
// of other BBoxRender fields.
pub trait BBoxRender {
  fn render<'a>(&'a self) -> ValueOrBBox<'a>;
}

impl<T: Serialize> BBoxRender for BBox<T> {
  fn render<'a>(&'a self) -> ValueOrBBox<'a> {
    ValueOrBBox::BBox(BBox::new(self.safe_unbox()))
  }
}

impl<T: Serialize> BBoxRender for T {
  fn render<'a>(&'a self) -> ValueOrBBox<'a> {
    ValueOrBBox::Serialize(self)
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
