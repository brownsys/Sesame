use std::collections::HashMap;
use std::any::Any;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use itertools::Itertools;

use crate::bbox::{BBox};
use crate::policy::{AnyPolicy, Policy};

pub fn compose_policies(policy1: Result<Option<AnyPolicy>, ()>, policy2: Result<Option<AnyPolicy>, ()>) -> Result<Option<AnyPolicy>, ()> {
    let policy1 = policy1?;
    let policy2 = policy2?;
    match (policy1, policy2) {
        (None, policy2) => Ok(policy2),
        (policy1, None) => Ok(policy1),
        (Some(policy1), Some(policy2)) =>
            Ok(Some(policy1.join(policy2)?)),
    }
}

// This provides a generic representation for values, bboxes, vectors, and structs mixing them.
pub enum AlohomoraTypeEnum {
    BBox(BBox<Box<dyn Any>, AnyPolicy>),
    Value(Box<dyn Any>),
    Vec(Vec<AlohomoraTypeEnum>),
    Struct(HashMap<String, AlohomoraTypeEnum>),
}

impl AlohomoraTypeEnum {
    // Combines the policies of all the BBox inside this type.
    pub fn policy(&self) -> Result<Option<AnyPolicy>, ()> {
        match self {
            AlohomoraTypeEnum::Value(_) => Ok(None),
            AlohomoraTypeEnum::BBox(bbox) => {
                Ok(Some(bbox.policy().clone()))
            },
            AlohomoraTypeEnum::Vec(vec)  => {
                vec
                    .into_iter()
                    .map(|e| e.policy())
                    .reduce(compose_policies)
                    .unwrap_or(Ok(None))
            }
            AlohomoraTypeEnum::Struct(hashmap) => {
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
    pub(crate) fn remove_bboxes(self) -> AlohomoraTypeEnum {
        match self {
            AlohomoraTypeEnum::Value(val) => AlohomoraTypeEnum::Value(val),
            AlohomoraTypeEnum::BBox(bbox) => AlohomoraTypeEnum::Value(bbox.consume().0),
            AlohomoraTypeEnum::Vec(vec) => AlohomoraTypeEnum::Vec(
                vec
                    .into_iter()
                    .map(|e| e.remove_bboxes())
                    .collect()
            ),
            AlohomoraTypeEnum::Struct(hashmap) => AlohomoraTypeEnum::Struct(
                hashmap
                    .into_iter()
                    .map(|(key, val)| (key, val.remove_bboxes()))
                    .collect(),
            ),
        }
    }

    // Coerces self into the given type provided it is a Value(...) of that type.
    pub fn coerce<T: 'static>(self) -> Result<T, ()> {
        match self {
            AlohomoraTypeEnum::Value(v) => match v.downcast() {
                Ok(t) => Ok(*t),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}

// Public: client code should derive this for structs that they want to unbox, fold, or pass to
// sandboxes.
pub trait AlohomoraType {
    type Out;     // Unboxed form of struct
    fn to_enum(self) -> AlohomoraTypeEnum;
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()>;
}

// Implement AlohomoraType for various primitives.
macro_rules! alohomora_type_impl {
    ($T: ty) => {
        #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl AlohomoraType for $T {
            type Out = $T;
            fn to_enum(self) -> AlohomoraTypeEnum{
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

// Implement AlohomoraType for Option
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: AlohomoraType> AlohomoraType for Option<T> {
    type Out = Option<T::Out>;
    fn to_enum(self) -> AlohomoraTypeEnum {
        match self {
            None => AlohomoraTypeEnum::Vec(Vec::new()),
            Some(t) => AlohomoraTypeEnum::Vec(vec![t.to_enum()]),
        }
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Vec(mut vec) => match vec.pop() {
                None => Ok(None),
                Some(v) => Ok(Some(T::from_enum(v)?)),
            },
            _ => Err(()),
        }
    }
}

// Implement AlohomoraType for BBox<T, P>
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: 'static, P: Policy + Clone + 'static> AlohomoraType for BBox<T, P> {
    type Out = T;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::BBox(self.into_any())
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

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<S: AlohomoraType> AlohomoraType for Vec<S> {
    type Out = Vec<S::Out>;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::Vec(self.into_iter().map(|s| s.to_enum()).collect())
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Vec(v) => {
                let mut result = Vec::with_capacity(v.len());
                for e in v.into_iter() {
                    result.push(S::from_enum(e)?);
                }
                Ok(result)
            }
            _ => Err(()),
        }
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<K: ToString + FromStr + Hash + Eq, S: AlohomoraType> AlohomoraType for HashMap<K, S> {
    type Out = HashMap<K, S::Out>;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::Struct(self.into_iter().map(|(k, v)| (k.to_string(), v.to_enum())).collect())
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Struct(m) => {
                let mut result = HashMap::new();
                for (k, v) in m.into_iter() {
                    match K::from_str(&k) {
                        Ok(k) => {
                            result.insert(k, S::from_enum(v)?);
                        },
                        Err(_) => {
                            return Err(())
                        }
                    }
                }
                Ok(result)
            }
            _ => Err(()),
        }
    }
}


#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl AlohomoraType for () {
    type Out = ();
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::Value(Box::new(()))
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Value(_) => Ok(()),
            _ => Err(()),
        }
    }
}

// Implement AlohomoraType for tuples made up of AlohomoraTypes.
macro_rules! alohomora_type_tuple_impl {
  ($([$A:tt,$i:tt]),*) => (
    #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
    impl<$($A: AlohomoraType,)*> AlohomoraType for ($($A,)*) {
        type Out = ($($A::Out,)*);
        fn to_enum(self) -> AlohomoraTypeEnum {
            #[allow(non_snake_case)]
            let ($($A,)*) = ($(self.$i.to_enum(),)*);
            AlohomoraTypeEnum::Vec(vec![$($A,)*])
        }
        fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
            match e {
                AlohomoraTypeEnum::Vec(v) => {
                    #[allow(non_snake_case)]
                    let ($($A,)*) = v.into_iter().collect_tuple().unwrap();
                    Ok(($($A::from_enum($A)?,)*))
                },
                _ => Err(()),
            }
        }
    }
  );
}

alohomora_type_tuple_impl!([A, 0]);
alohomora_type_tuple_impl!([A, 0], [B, 1]);
alohomora_type_tuple_impl!([A, 0], [B, 1], [C, 2]);
alohomora_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3]);
alohomora_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4]);
alohomora_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4], [F, 5]);
alohomora_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4], [F, 5], [G, 6]);
alohomora_type_tuple_impl!(
    [A, 0],
    [B, 1],
    [C, 2],
    [D, 3],
    [E, 4],
    [F, 5],
    [G, 6],
    [H, 7]
);
alohomora_type_tuple_impl!(
    [A, 0],
    [B, 1],
    [C, 2],
    [D, 3],
    [E, 4],
    [F, 5],
    [G, 6],
    [H, 7],
    [I, 8]
);
alohomora_type_tuple_impl!(
    [A, 0],
    [B, 1],
    [C, 2],
    [D, 3],
    [E, 4],
    [F, 5],
    [G, 6],
    [H, 7],
    [I, 8],
    [J, 9]
);alohomora_type_tuple_impl!(
    [A, 0],
    [B, 1],
    [C, 2],
    [D, 3],
    [E, 4],
    [F, 5],
    [G, 6],
    [H, 7],
    [I, 8],
    [J, 9],
    [K, 10]
);
alohomora_type_tuple_impl!(
    [A, 0],
    [B, 1],
    [C, 2],
    [D, 3],
    [E, 4],
    [F, 5],
    [G, 6],
    [H, 7],
    [I, 8],
    [J, 9],
    [K, 10],
    [L, 11]
);

// Implement AlohomoraType for Arc<Mutex<T>>
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: AlohomoraType> AlohomoraType for Arc<Mutex<T>> {
    type Out = Arc<Mutex<T::Out>>;
    fn to_enum(self) -> AlohomoraTypeEnum {
        let t = Arc::into_inner(self).unwrap().into_inner().unwrap();
        AlohomoraTypeEnum::Vec(vec![t.to_enum()])
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Vec(mut v) => {
                let t = v.pop().unwrap();
                Ok(Arc::new(Mutex::new(T::from_enum(t)?)))
            }
            _ => Err(()),
        }
    }
}
