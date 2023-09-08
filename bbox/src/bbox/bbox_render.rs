extern crate erased_serde;
extern crate figment;

use std::any::Any;
use std::collections::{BTreeMap, HashMap};

use erased_serde::Serialize;
use figment::value::Value as FValue;

// Our BBox struct.
use crate::bbox::{BBox, EitherBBox};
use crate::context::Context;
use crate::policy::Policy;

// Types for cheap references of BBox with type erasure.
pub mod refs {
    use crate::policy::AnyPolicy;
    use super::{Any, BBox, Policy, Serialize};
    // AnyPolicy (by ref)
    pub struct RefPolicy<'a> {
        policy: &'a dyn Policy,
    }

    impl<'a> RefPolicy<'a> {
        fn new(policy: &'a dyn Policy) -> Self {
            RefPolicy { policy }
        }
    }

    impl<'a> Policy for RefPolicy<'a> {
        fn name(&self) -> String {
            format!("RefPolicy({})", self.policy.name())
        }
        fn check(&self, context: &dyn Any) -> bool {
            self.policy.check(context)
        }
    }

    pub struct RefBBox<'a>(pub(super) BBox<&'a dyn Serialize, RefPolicy<'a>>);
    impl<'a, T: Serialize, P: Policy> From<&'a BBox<T, P>> for RefBBox<'a> {
        fn from(value: &'a BBox<T, P>) -> Self {
            RefBBox(BBox::new(&value.t, RefPolicy::new(&value.p)))
        }
    }
    impl<'a> RefBBox<'a> {
        pub fn get(&self) -> &BBox<&'a dyn Serialize, RefPolicy<'a>> {
            &self.0
        }
    }
}

// A BBox with type T erased, a primitive value, or a collection of mixed-type
// values.
pub enum Renderable<'a> {
    BBox(refs::RefBBox<'a>),
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
                let t = bbox.0.unbox(context);
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
            Renderable::BBox(bbox) => Ok(bbox.0.t),
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
impl<T: Serialize, P: Policy + Clone> BBoxRender for BBox<T, P> {
    fn render(&self) -> Renderable {
        Renderable::BBox(refs::RefBBox::from(self))
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
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn make_test_context() -> Context<String, String> {
        Context::new(None, String::from(""))
    }

    #[test]
    fn test_renderable_serialize() {
        let string = String::from("my test!");
        let renderable = string.render();
        assert!(matches!(renderable, Renderable::Serialize(_)));
        let result = renderable.transform(&make_test_context());
        assert!(matches!(result, Result::Ok(FValue::String(_, result)) if result == string));
    }

    #[test]
    fn test_renderable_bbox() {
        let bbox = BBox::new(String::from("my bbox!"), NoPolicy {});
        let renderable = bbox.render();
        assert!(matches!(renderable, Renderable::BBox(_)));
        let result = renderable.transform(&make_test_context());
        assert!(matches!(result, Result::Ok(FValue::String(_, result)) if result == bbox.t));
    }

    #[test]
    fn test_renderable_either() {
        let either: EitherBBox<String, NoPolicy> = EitherBBox::Value(String::from("my_test!"));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::Serialize(_)));
        let result = renderable.transform(&make_test_context());
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == String::from("my_test!"))
        );

        let either = EitherBBox::BBox(BBox::new(String::from("my_bbox!"), NoPolicy {}));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::BBox(_)));
        let result = renderable.transform(&make_test_context());
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
        let result = renderable.transform(&make_test_context());
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
        let result = renderable.transform(&make_test_context());
        assert!(matches!(result, Result::Ok(FValue::Dict(_, _))));
        if let Result::Ok(FValue::Dict(_, dict)) = result {
            assert!(matches!(dict.get("key1"), Option::Some(FValue::String(_, e)) if e == "val1"));
            assert!(matches!(dict.get("key2"), Option::Some(FValue::String(_, e)) if e == "val2"));
        }
    }
}
