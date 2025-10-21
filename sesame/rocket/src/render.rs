extern crate erased_serde;
extern crate figment;

use erased_serde::Serialize;
use figment::value::Value as FValue;
use std::collections::{BTreeMap, HashMap};

// Our BBox struct.
use sesame::bbox::{BBox, EitherBBox};
use sesame::extensions::{
    ExtensionContext, SesameExtension, SesameRefExtension, UncheckedSesameExtension,
};
use sesame::policy::{Policy, Reason, RefPolicy};

use crate::error::SesameRenderResult;
#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::BBoxRender;

// Types for cheap references of BBox with type erasure.
type RefBBox<'a> = BBox<&'a dyn Serialize, RefPolicy<'a, dyn Policy + 'a>>;

// Sesame extension that performs policy check then renders template if successful.
struct RenderPolicyChecker {}
impl<'a> SesameExtension<&'a dyn Serialize, RefPolicy<'a, dyn Policy + 'a>, figment::Result<FValue>>
    for RenderPolicyChecker
{
    fn apply(
        &mut self,
        data: &'a dyn Serialize,
        _policy: RefPolicy<'a, dyn Policy + 'a>,
    ) -> figment::Result<FValue> {
        FValue::serialize(data)
    }
}

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
        self,
        template: &str,
        context: &ExtensionContext,
    ) -> SesameRenderResult<FValue> {
        match self {
            Renderable::BBox(bbox) => {
                let mut checker = RenderPolicyChecker {};
                let reason = Reason::TemplateRender(template);
                Ok(bbox.checked_extension(&mut checker, context, reason)??)
            }
            Renderable::Serialize(obj) => Ok(FValue::serialize(obj)?),
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
impl<T: Serialize, P: Policy> BBoxRender for BBox<T, P> {
    fn render(&self) -> Renderable {
        struct Converter {}
        impl UncheckedSesameExtension for Converter {}
        impl<'a, T: Serialize, P: Policy> SesameRefExtension<'a, T, P, Renderable<'a>> for Converter {
            fn apply_ref(&mut self, data: &'a T, policy: &'a P) -> Renderable<'a> {
                Renderable::BBox(BBox::new(data, RefPolicy::new(policy)))
            }
        }
        self.unchecked_extension_ref(&mut Converter {})
    }
}

// Auto implement BBoxRender for EitherBBox.
impl<T: Serialize, P: Policy> BBoxRender for EitherBBox<T, P> {
    fn render(&self) -> Renderable {
        match self {
            EitherBBox::Left(value) => Renderable::Serialize(value),
            EitherBBox::Right(bbox) => bbox.render(),
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
    use super::*;
    use sesame::context::Context;
    use sesame::policy::NoPolicy;

    #[test]
    fn test_renderable_serialize() {
        let string = String::from("my test!");
        let renderable = string.render();
        assert!(matches!(renderable, Renderable::Serialize(_)));
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
        assert!(matches!(result, Result::Ok(FValue::String(_, result)) if result == string));
    }

    #[test]
    fn test_renderable_bbox() {
        let bbox = BBox::new(String::from("my bbox!"), NoPolicy {});
        let renderable = bbox.render();
        assert!(matches!(renderable, Renderable::BBox(_)));
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == bbox.discard_box())
        );
    }

    #[test]
    fn test_renderable_either() {
        let either: EitherBBox<String, NoPolicy> = EitherBBox::Left(String::from("my_test!"));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::Serialize(_)));
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == String::from("my_test!"))
        );

        let either = EitherBBox::Right(BBox::new(String::from("my_bbox!"), NoPolicy {}));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::BBox(_)));
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
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
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
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
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
        assert!(matches!(result, Result::Ok(FValue::Dict(_, _))));
        if let Result::Ok(FValue::Dict(_, dict)) = result {
            assert!(matches!(dict.get("key1"), Option::Some(FValue::String(_, e)) if e == "val1"));
            assert!(matches!(dict.get("key2"), Option::Some(FValue::String(_, e)) if e == "val2"));
        }
    }
}
