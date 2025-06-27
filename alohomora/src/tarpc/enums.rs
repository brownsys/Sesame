use crate::bbox::BBox;
use crate::policy::TahiniPolicy;
use crate::tarpc::traits::{TahiniError, TahiniType};
use serde::ser::{SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTupleVariant};
use std::collections::HashMap;

pub enum TahiniEnum {
    Value(Box<dyn erased_serde::Serialize>),
    BBox(BBox<Box<dyn erased_serde::Serialize>, TahiniPolicy>),
    Vec(Vec<TahiniEnum>),
    Struct(&'static str, HashMap<&'static str, TahiniEnum>),
    Enum(&'static str, u32, &'static str, TahiniVariantsEnum),
    Option(Option<Box<TahiniEnum>>),
    Result(Result<Box<TahiniEnum>, Box<dyn TahiniError>>),
}

pub enum TahiniVariantsEnum {
    Unit,
    Struct(HashMap<&'static str, TahiniEnum>),
    NewType(Box<TahiniEnum>),
    Tuple(Vec<TahiniEnum>),
}

fn serialize_enum<S: serde::Serializer>(
    enum_name: &'static str,
    index: &u32,
    variant_name: &'static str,
    variant: &TahiniVariantsEnum,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match variant {
        TahiniVariantsEnum::Unit => {
            serializer.serialize_unit_variant(enum_name, *index, variant_name)
        }
        TahiniVariantsEnum::Struct(map) => {
            let mut struct_ser =
                serializer.serialize_struct_variant(enum_name, *index, variant_name, map.len())?;
            for (k, v) in map.iter() {
                struct_ser.serialize_field(k, &PrivEnumWrapper(v))?;
            }
            struct_ser.end()
        }
        TahiniVariantsEnum::NewType(inner) => serializer.serialize_newtype_variant(
            enum_name,
            *index,
            variant_name,
            &PrivEnumWrapper(&(*inner)),
        ),
        TahiniVariantsEnum::Tuple(iter) => {
            let mut tuple_ser =
                serializer.serialize_tuple_variant(enum_name, *index, variant_name, iter.len())?;
            for e in iter.iter() {
                tuple_ser.serialize_field(&PrivEnumWrapper(e))?;
            }
            tuple_ser.end()
        }
    }
}

//Private struct for specific TahiniEnum leaves.
struct PrivEnumWrapper<'a>(&'a TahiniEnum);
impl<'a> serde::Serialize for PrivEnumWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            TahiniEnum::Value(val) => erased_serde::serialize(&val, serializer),
            TahiniEnum::BBox(bbox) => {
                let t = bbox.data();
                let p = bbox.policy();
                // let mut bbox_ser = erased_serde::serialize(value, serializer)
                let mut bbox_ser = serializer.serialize_struct("BBox", 2)?;
                bbox_ser.serialize_field("fb", &(*t))?;
                bbox_ser.serialize_field("p", &p)?;
                bbox_ser.end()
            }
            TahiniEnum::Vec(vec) => {
                let mut vec_ser = serializer.serialize_seq(Some(vec.len()))?;
                for e in vec.iter() {
                    vec_ser.serialize_element(&PrivEnumWrapper(e))?;
                }
                vec_ser.end()
            }
            TahiniEnum::Struct(struct_name, map) => {
                let mut struct_ser = serializer.serialize_struct(struct_name, map.len())?;
                for (k, v) in map.iter() {
                    struct_ser.serialize_field(k, &PrivEnumWrapper(v))?;
                }
                struct_ser.end()
            }
            TahiniEnum::Enum(enum_name, idx, name, val) => {
                serialize_enum(*enum_name, idx, *name, val, serializer)
            }
            TahiniEnum::Option(opt) => match opt {
                None => None::<PrivEnumWrapper>.serialize(serializer),
                Some(val) => Some(PrivEnumWrapper(&val)).serialize(serializer),
            },
            TahiniEnum::Result(res) => res
                .as_ref()
                .map(|v| PrivEnumWrapper(v))
                .serialize(serializer),
        }
    }
}

//Only messy part here is to have two different wrappers operating the same function
//The real reason is that it gives explicit typing to our structs.
//It's open to debate whether we want to have two layers :shrug:

pub struct TahiniSafeWrapper<T: TahiniType>(pub(crate) T);
impl<T: TahiniType + Sized> serde::Serialize for TahiniSafeWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        PrivEnumWrapper(&self.0.to_tahini_enum()).serialize(serializer)
    }
}
