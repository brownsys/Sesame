extern crate erased_serde;
extern crate figment;

use erased_serde::Serialize;
use figment::value::Value as FValue;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

// Our PCon struct.
use sesame::extensions::{
    ExtensionContext, SesameExtension, SesameRefExtension, UncheckedSesameExtension,
};
use sesame::pcon::{EitherPCon, PCon};
use sesame::policy::{Policy, Reason, RefPolicy};

use crate::error::SesameRenderResult;

#[cfg(feature = "derive")]
pub use sesame_derive::PConRender;

// Types for cheap references of PCon with type erasure.
type RefPCon<'a> = PCon<&'a dyn Serialize, RefPolicy<'a, dyn Policy + 'a>>;

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

// A PCon with type T erased, a primitive value, or a collection of mixed-type
// values.
pub enum Renderable<'a> {
    PCon(RefPCon<'a>),
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
            Renderable::PCon(pcon) => {
                let mut checker = RenderPolicyChecker {};
                let reason = Reason::TemplateRender(template);
                Ok(pcon.checked_extension(&mut checker, context, reason)??)
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
// The `sesame_derive` lib provides macros to derive this for structs that consist
// of other PConRender fields.
pub trait PConRender {
    fn render(&self) -> Renderable;
}

// Helper used by the PConRender derive macro to dispatch field rendering.
// The inherent impl (T: PConRender) wins over the trait impl (T: Serialize)
// because Rust always prefers inherent methods over trait methods on the same
// receiver type. This gives PConRender priority without needing specialization.
pub struct RenderFieldHelper<'a, T>(pub &'a T);

impl<'a, T: PConRender> RenderFieldHelper<'a, T> {
    pub fn render_field(&self) -> Renderable<'a> {
        self.0.render()
    }
}

pub trait SerializeFieldFallback<'a> {
    fn render_field(&self) -> Renderable<'a>;
}
impl<'a, T: Serialize> SerializeFieldFallback<'a> for RenderFieldHelper<'a, T> {
    fn render_field(&self) -> Renderable<'a> {
        Renderable::Serialize(self.0)
    }
}

// Auto implement PConRender for unboxed primitive types.
macro_rules! render_serialize_impl {
    ($($T:ty),+) => {
        $(
            impl PConRender for $T {
                fn render<'a>(&'a self) -> Renderable<'a> {
                    Renderable::Serialize(self)
                }
            }
        )+
    };
}
render_serialize_impl!(
    String, &str,
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
    bool, char,
    serde_json::Value,
    chrono::NaiveDate,
    chrono::NaiveDateTime,
    chrono::NaiveTime,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Local>,
    chrono::DateTime<chrono::FixedOffset>
);

// Auto implement PConRender for PCon.
impl<T: Serialize, P: Policy> PConRender for PCon<T, P> {
    fn render(&self) -> Renderable {
        struct Converter {}
        impl UncheckedSesameExtension for Converter {}
        impl<'a, T: Serialize, P: Policy> SesameRefExtension<'a, T, P, Renderable<'a>> for Converter {
            fn apply_ref(&mut self, data: &'a T, policy: &'a P) -> Renderable<'a> {
                Renderable::PCon(PCon::new(data, RefPolicy::new(policy)))
            }
        }
        self.unchecked_extension_ref(&mut Converter {})
    }
}

// Auto implement PConRender for EitherPCon.
impl<T: Serialize, P: Policy> PConRender for EitherPCon<T, P> {
    fn render(&self) -> Renderable {
        match self {
            EitherPCon::Left(value) => Renderable::Serialize(value),
            EitherPCon::Right(pcon) => pcon.render(),
        }
    }
}

// Auto implement PConRender for Vec.
impl<T: PConRender> PConRender for Vec<T> {
    fn render(&self) -> Renderable {
        Renderable::Array(self.iter().map(|v| v.render()).collect())
    }
}

// Auto implement PConRender for HashMap.
impl<T: PConRender> PConRender for HashMap<&str, T> {
    fn render(&self) -> Renderable {
        let mut map = BTreeMap::new();
        for (key, val) in self.iter() {
            map.insert((*key).into(), val.render());
        }
        Renderable::Dict(map)
    }
}

impl<T: PConRender> PConRender for HashMap<String, T> {
    fn render(&self) -> Renderable {
        let mut map = BTreeMap::new();
        for (key, val) in self.iter() {
            map.insert(key.clone(), val.render());
        }
        Renderable::Dict(map)
    }
}

impl<T: PConRender> PConRender for Option<T> {
    fn render(&self) -> Renderable {
        match self {
            Some(v) => v.render(),
            None => {
                static NULL: Option<()> = None;
                Renderable::Serialize(&NULL)
            }
        }
    }
}

impl<T: PConRender> PConRender for &T {
    fn render(&self) -> Renderable {
        (**self).render()
    }
}

impl<T: PConRender> PConRender for &mut T {
    fn render(&self) -> Renderable {
        (**self).render()
    }
}

impl<T: PConRender> PConRender for Box<T> {
    fn render(&self) -> Renderable {
        (**self).render()
    }
}

impl<T: PConRender> PConRender for Arc<T> {
    fn render(&self) -> Renderable {
        (**self).render()
    }
}

impl PConRender for () {
    fn render(&self) -> Renderable {
        Renderable::Serialize(self)
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
    fn test_renderable_pcon() {
        let pcon = PCon::new(String::from("my pcon!"), NoPolicy {});
        let renderable = pcon.render();
        assert!(matches!(renderable, Renderable::PCon(_)));
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == pcon.discard_box())
        );
    }

    #[test]
    fn test_renderable_either() {
        let either: EitherPCon<String, NoPolicy> = EitherPCon::Left(String::from("my_test!"));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::Serialize(_)));
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == String::from("my_test!"))
        );

        let either = EitherPCon::Right(PCon::new(String::from("my_pcon!"), NoPolicy {}));
        let renderable = either.render();
        assert!(matches!(renderable, Renderable::PCon(_)));
        let context = ExtensionContext::new(Context::test(()));
        let result = renderable.transform("", &context);
        assert!(
            matches!(result, Result::Ok(FValue::String(_, result)) if result == String::from("my_pcon!"))
        );
    }

    #[test]
    fn test_renderable_vec() {
        let mut vec = Vec::new();
        vec.push(PCon::new(String::from("hello"), NoPolicy {}));
        vec.push(PCon::new(String::from("bye"), NoPolicy {}));
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
        map.insert("key1", PCon::new(String::from("val1"), NoPolicy {}));
        map.insert("key2", PCon::new(String::from("val2"), NoPolicy {}));
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
