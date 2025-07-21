use std::any::Any;
use std::collections::HashMap;
use crate::bbox::BBox;
use crate::policy::AnyPolicy;
use crate::sesame_type::dyns::{SesameDynType, SesameTypeDynTypes};
use crate::sesame_type::helpers::compose_policies;

// This provides a generic representation for values, bboxes, vectors, and structs mixing them.
pub enum SesameTypeEnumDyn<T: SesameDynType + ?Sized> {
    BBox(BBox<Box<T>, AnyPolicy>),
    Value(Box<T>),
    Vec(Vec<SesameTypeEnumDyn<T>>),
    Struct(HashMap<String, SesameTypeEnumDyn<T>>),
}

impl<T: SesameDynType + ?Sized> SesameTypeEnumDyn<T> {
    // Combines the policies of all the BBox inside this type.
    pub fn policy(&self) -> Result<Option<AnyPolicy>, ()> {
        match self {
            SesameTypeEnumDyn::Value(_) => Ok(None),
            SesameTypeEnumDyn::BBox(bbox) => Ok(Some(bbox.policy().clone())),
            SesameTypeEnumDyn::Vec(vec) => vec
                .into_iter()
                .map(|e| e.policy())
                .reduce(compose_policies)
                .unwrap_or(Ok(None)),
            SesameTypeEnumDyn::Struct(hashmap) => {
                // iterate over hashmap, collect policies
                hashmap
                    .into_iter()
                    .map(|(_, e)| e.policy())
                    .reduce(compose_policies)
                    .unwrap_or(Ok(None))
            }
        }
    }

    // Transforms the Enum to an unboxed enum.
    pub(crate) fn remove_bboxes(self) -> Self {
        match self {
            SesameTypeEnumDyn::Value(val) => SesameTypeEnumDyn::Value(val),
            SesameTypeEnumDyn::BBox(bbox) => SesameTypeEnumDyn::Value(bbox.consume().0),
            SesameTypeEnumDyn::Vec(vec) => {
                SesameTypeEnumDyn::Vec(vec.into_iter().map(|e| e.remove_bboxes()).collect())
            }
            SesameTypeEnumDyn::Struct(hashmap) => SesameTypeEnumDyn::Struct(
                hashmap
                    .into_iter()
                    .map(|(key, val)| (key, val.remove_bboxes()))
                    .collect(),
            ),
        }
    }

    // Coerces self into the given type provided it is a Value(...) of that type.
    pub fn coerce<R: Any>(self) -> Result<R, ()> where T: SesameTypeDynTypes<R> {
        match self {
            SesameTypeEnumDyn::Value(v) => match v.upcast_box().downcast() {
                Ok(t) => Ok(*t),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}

// Alias for ease of use.
pub type SesameTypeEnum = SesameTypeEnumDyn<dyn Any>;