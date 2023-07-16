extern crate erased_serde;
extern crate figment;

use std::collections::{BTreeMap, HashMap};

use erased_serde::Serialize;
use figment::value::Value as FValue;

// Our BBox struct.
use crate::bbox::{BBox, EitherBBox};
use crate::policy::Context;

// A BBox with type T erased, a primitive value, or a collection of mixed-type
// values.
pub enum Renderable<'a> {
    BBox(BBox<&'a dyn Serialize>),
    Serialize(&'a dyn Serialize),
    Dict(BTreeMap<String, Renderable<'a>>),
    Array(Vec<Renderable<'a>>),
}

impl<'a> Renderable<'a> {
    pub(crate) fn transform<U: 'static, D: 'static>(
        &self,
        context: &Context<U, D>,
    ) -> Result<FValue, figment::Error> {
        match self {
            Renderable::BBox(bbox) => {
                let t = bbox.unbox(context);
                FValue::serialize(t)
            }
            Renderable::Serialize(obj) => FValue::serialize(obj),
            Renderable::Dict(map) => {
                let mut tmap: BTreeMap<String, FValue> = BTreeMap::new();
                for (k, v) in map {
                    let v = v.transform(context)?;
                    tmap.insert(k.clone(), v);
                }
                Ok(FValue::from(tmap))
            }
            Renderable::Array(vec) => {
                let mut tvec: Vec<FValue> = Vec::new();
                for v in vec {
                    let v = v.transform(context)?;
                    tvec.push(v);
                }
                Ok(FValue::from(tvec))
            }
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
    fn render(&self) -> Renderable;
}

// Auto implement BBoxRender for unboxed primitive types.
macro_rules! render_serialize_impl {
    ($T: ty) => {
        impl BBoxRender for $T {
            fn render<'a>(&'a self) -> Renderable<'a> {
                Renderable::Serialize(self)
            }
        }
    };
}
render_serialize_impl!(String);
render_serialize_impl!(u64);
render_serialize_impl!(i64);
render_serialize_impl!(u8);
render_serialize_impl!(i8);

// Auto implement BBoxRender for BBox.
impl<T: Serialize> BBoxRender for BBox<T> {
    fn render(&self) -> Renderable {
        Renderable::BBox(self.map(|t| t as &dyn Serialize))
    }
}

// Auto implement BBoxRender for EitherBBox.
impl<T: Serialize> BBoxRender for EitherBBox<T> {
    fn render(&self) -> Renderable {
        match self {
            EitherBBox::Value(value) => Renderable::Serialize(value),
            EitherBBox::BBox(bbox) => bbox.render(),
        }
    }
}

// Auto implement BBoxRender for Vec.
impl<T: BBoxRender> BBoxRender for Vec<T> {
    fn render(&self) -> Renderable {
        Renderable::Array(self.iter().map(|v| v.render()).collect())
    }
}

// Auto implement BBoxRender for HashMap.
impl<T: BBoxRender> BBoxRender for HashMap<&str, T> {
    fn render(&self) -> Renderable {
        let mut map = BTreeMap::new();
        for (key, val) in self.iter() {
            map.insert((*key).into(), val.render());
        }
        Renderable::Dict(map)
    }
}
