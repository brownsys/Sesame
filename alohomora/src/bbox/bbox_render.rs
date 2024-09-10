extern crate erased_serde;
extern crate figment;

use std::collections::{BTreeMap, HashMap};

use erased_serde::Serialize;
use figment::value::Value as FValue;

// Our BBox struct.
use crate::bbox::{BBox, EitherBBox};
use crate::context::UnprotectedContext;
use crate::policy::{Policy, Reason, RefPolicy};

// Types for cheap references of BBox with type erasure.
type RefBBox<'a> = BBox<&'a dyn Serialize, RefPolicy<'a, dyn Policy + 'a>>;

// A BBox with type T erased, a primitive value, or a collection of mixed-type
// values.
pub enum Renderable<'a> {
    BBox(RefBBox<'a>),
    Serialize(&'a dyn Serialize),
    Dict(BTreeMap<String, Renderable<'a>>),
    Array(Vec<Renderable<'a>>),
}

impl<'a> Renderable<'a> {
    pub(crate) fn transform(
        &self,
        template: &str,
        context: &UnprotectedContext,
    ) -> Result<FValue, figment::Error> {
        match self {
            Renderable::BBox(bbox) => {
                if bbox.policy().check(context, Reason::TemplateRender(template)) {
                    FValue::serialize(*bbox.data())
                } else {
                    Err(figment::Error::from(format!("Policy check failed {}", bbox.policy().name())))
                }
            }
            Renderable::Serialize(obj) => FValue::serialize(obj),
            Renderable::Dict(map) => {
                let mut tmap: BTreeMap<String, FValue> = BTreeMap::new();
                for (k, v) in map {
                    let v = v.transform(template, context)?;
                    tmap.insert(k.clone(), v);
                }
                Ok(FValue::from(tmap))
            }
            Renderable::Array(vec) => {
                let mut tvec: Vec<FValue> = Vec::new();
                for v in vec {
                    let v = v.transform(template, context)?;
                    tvec.push(v);
                }
                Ok(FValue::from(tvec))
            }
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
render_serialize_impl!(bool);

// Auto implement BBoxRender for BBox.
impl<T: Serialize, P: Policy + Clone> BBoxRender for BBox<T, P> {
    fn render(&self) -> Renderable {
        Renderable::BBox(BBox::new(self.data(), RefPolicy::new(self.policy())))
    }
}

// Auto implement BBoxRender for EitherBBox.
impl<T: Serialize, P: Policy + Clone> BBoxRender for EitherBBox<T, P> {
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

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::policy::NoPolicy;
    use super::*;

    #[test]
    fn test_renderable_serialize() {
        let string = String::from("my test!");
        let renderable = string.render();
        assert!(matches!(renderable, Renderable::Serialize(_)));
        let result = renderable.transform("", &UnprotectedContext::test(()));
        assert!(matches!(result, Result::Ok(FValue::String(_, result)) if result == string));
    }

    #[test]
    fn test_renderable_bbox() {
        let bbox = BBox::new(String::from("my bbox!"), NoPolicy {});
        let renderable = bbox.render();
        assert!(matches!(renderable, Renderable::BBox(_)));
        let result = renderable.transform("", &UnprotectedContext::test(()));
        assert!(matches!(result, Result::Ok(FValue::String(_, result)) if &result == bbox.data()));
    }

    #[test]
    fn test_renderable_either() {
        let either: EitherBBox<String, NoPolicy> = EitherBBox::Value(String::from("my_test!"));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::Serialize(_)));
        let result = renderable.transform("", &UnprotectedContext::test(()));
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == String::from("my_test!"))
        );

        let either = EitherBBox::BBox(BBox::new(String::from("my_bbox!"), NoPolicy {}));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::BBox(_)));
        let result = renderable.transform("", &UnprotectedContext::test(()));
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == String::from("my_bbox!"))
        );
    }

    #[test]
    fn test_renderable_vec() {
        let mut vec = Vec::new();
        vec.push(BBox::new(String::from("hello"), NoPolicy {}));
        vec.push(BBox::new(String::from("bye"), NoPolicy {}));
        let renderable = vec.render();
        assert!(matches!(renderable, Renderable::Array(_)));
        let result = renderable.transform("", &UnprotectedContext::test(()));
        assert!(matches!(result, Result::Ok(FValue::Array(_, _))));
        if let Result::Ok(FValue::Array(_, arr)) = result {
            assert!(matches!(&arr[0], FValue::String(_, e) if e == "hello"));
            assert!(matches!(&arr[1], FValue::String(_, e) if e == "bye"));
        }
    }

    #[test]
    fn test_renderable_map() {
        let mut map = HashMap::new();
        map.insert("key1", BBox::new(String::from("val1"), NoPolicy {}));
        map.insert("key2", BBox::new(String::from("val2"), NoPolicy {}));
        let renderable = map.render();
        assert!(matches!(renderable, Renderable::Dict(_)));
        let result = renderable.transform("", &UnprotectedContext::test(()));
        assert!(matches!(result, Result::Ok(FValue::Dict(_, _))));
        if let Result::Ok(FValue::Dict(_, dict)) = result {
            assert!(matches!(dict.get("key1"), Option::Some(FValue::String(_, e)) if e == "val1"));
            assert!(matches!(dict.get("key2"), Option::Some(FValue::String(_, e)) if e == "val2"));
        }
    }
}
