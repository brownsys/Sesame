use std::collections::HashMap;
use std::any::Any;

use crate::bbox::BBox;
use crate::policy::{AnyPolicy, Policy};
use crate::r#type::{AlohomoraTypePolicy, compose_policies};

// This provides a generic representation for values, bboxes, vectors, and structs mixing them.
#[derive(Debug)]
pub enum AlohomoraTypeEnum {
    BBox(BBox<Box<dyn Any>, AnyPolicy>),
    Value(Box<dyn Any>),
    Vec(Vec<AlohomoraTypeEnum>),
    Struct(HashMap<String, AlohomoraTypeEnum>),
}

impl AlohomoraTypeEnum {
    // Combines the policies of all the BBox inside this type.
    pub fn policy(&self) -> AlohomoraTypePolicy {
        match self {
            AlohomoraTypeEnum::Value(_) => AlohomoraTypePolicy::NoPolicy,
            AlohomoraTypeEnum::BBox(bbox) => {
                AlohomoraTypePolicy::Policy(bbox.policy().clone())
            },
            AlohomoraTypeEnum::Vec(vec)  => {
                vec
                    .into_iter()
                    .map(|e| e.policy())
                    .reduce(compose_policies)
                    .unwrap_or(AlohomoraTypePolicy::NoPolicy)
            }
            AlohomoraTypeEnum::Struct(hashmap) => {
                // iterate over hashmap, collect policies
                hashmap
                    .into_iter()
                    .map(|(_, e)| e.policy())
                    .reduce(compose_policies)
                    .unwrap_or(AlohomoraTypePolicy::NoPolicy)
            }
        }
    }

    // Transforms the Enum to an unboxed enum.
    pub(crate) fn remove_bboxes(self) -> AlohomoraTypeEnum {
        match self {
            AlohomoraTypeEnum::Value(val) => AlohomoraTypeEnum::Value(val),
            AlohomoraTypeEnum::BBox(bbox) => AlohomoraTypeEnum::Value(bbox.t),
            AlohomoraTypeEnum::Vec(vec) => AlohomoraTypeEnum::Vec(
                vec
                    .into_iter()
                    .map(|e| e.remove_bboxes())
                    .collect()),
            AlohomoraTypeEnum::Struct(hashmap) => AlohomoraTypeEnum::Struct(
                hashmap
                    .into_iter()
                    .map(|(key, val)| (key, val.remove_bboxes()))
                    .collect(),
            ),
        }
    }
}

// Public: client code should derive this for structs that they want to unbox, fold, or pass to
// sandboxes.
pub trait AlohomoraType<P: Policy = AnyPolicy> {
    type Out;     // Unboxed form of struct
    fn to_enum(self) -> AlohomoraTypeEnum;
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()>;
}


// Implement AlohomoraType for various primitives.
macro_rules! alohomora_type_impl {
    ($T: ty) => {
        impl AlohomoraType for $T {
            type Out = $T;
            fn to_enum(self) -> AlohomoraTypeEnum {
                AlohomoraTypeEnum::Value(Box::new(self))
            }
            fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
                match e {
                    AlohomoraTypeEnum::Value(v) => match v.downcast() {
                        Err(_) => Err(()),
                        Ok(v) => Ok(*v),
                    },
                    _ => Err(()),
                }
            }
        }

    };
}

alohomora_type_impl!(i8);
alohomora_type_impl!(i16);
alohomora_type_impl!(i32);
alohomora_type_impl!(i64);
alohomora_type_impl!(u8);
alohomora_type_impl!(u16);
alohomora_type_impl!(u32);
alohomora_type_impl!(u64);
alohomora_type_impl!(bool);
alohomora_type_impl!(f64);
alohomora_type_impl!(String);

// Implement AlohomoraType for BBox<T, P>
impl<T: 'static, P: Policy + Clone + 'static> AlohomoraType for BBox<T, P> {
    type Out = T;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::BBox(self.to_any_type_and_policy())
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Value(v) => match v.downcast() {
                Err(_) => Err(()),
                Ok(v) => Ok(*v),
            },
            _ => Err(()),
        }
    }
}

// Implement AlohomoraType for containers of AlohomoraTypes
impl<S: AlohomoraType> AlohomoraType for Vec<S> {
    type Out = Vec<S::Out>;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::Vec(self.into_iter().map(|s| s.to_enum()).collect())
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Vec(v) => {
                let mut result = Vec::new();
                for e in v.into_iter() {
                    result.push(S::from_enum(e)?);
                }
                Ok(result)
            }
            _ => Err(()),
        }
    }
}

impl<S: AlohomoraType> AlohomoraType for HashMap<String, S> {
    type Out = HashMap<String, S::Out>;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::Struct(self.into_iter().map(|(k, v)| (k, v.to_enum())).collect())
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Struct(m) => {
                let mut result = HashMap::new();
                for (k, v) in m.into_iter() {
                    result.insert(k, S::from_enum(v)?);
                }
                Ok(result)
            }
            _ => Err(()),
        }
    }
}