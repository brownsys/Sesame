use itertools::Itertools;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::bbox::BBox;
use crate::policy::Policy;
use crate::sesame_type::r#enum::SesameTypeEnumDyn;
use crate::sesame_type::r#type::{SesameTypeDyn, AnySerialize};

// Implement SesameType for various primitives.
macro_rules! sesame_type_value_impl {
    ($T: ty) => {
        #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl SesameTypeDyn<dyn Any> for $T {
            type Out = $T;
            fn to_enum(self) -> SesameTypeEnumDyn<dyn Any> {
                SesameTypeEnumDyn::Value(Box::new(self))
            }
            fn from_enum(e: SesameTypeEnumDyn<dyn Any>) -> Result<Self::Out, ()> {
                match e {
                    SesameTypeEnumDyn::Value(v) => match v.downcast() {
                        Err(_) => Err(()),
                        Ok(v) => Ok(*v),
                    },
                    _ => Err(()),
                }
            }
        }
       #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl SesameTypeDyn<dyn AnySerialize> for $T {
            type Out = $T;
            fn to_enum(self) -> SesameTypeEnumDyn<dyn AnySerialize> {
                SesameTypeEnumDyn::Value(Box::new(self))
            }
            fn from_enum(e: SesameTypeEnumDyn<dyn AnySerialize>) -> Result<Self::Out, ()> {
                match e {
                    SesameTypeEnumDyn::Value(v) => {
                        match v.upcast_any_box().downcast() {
                            Ok(v) => Ok(*v),
                            Err(_) => Err(()),
                        }
                    },
                    _ => Err(()),
                }
            }
        }
    };
}

sesame_type_value_impl!(i8);
sesame_type_value_impl!(i16);
sesame_type_value_impl!(i32);
sesame_type_value_impl!(i64);
sesame_type_value_impl!(u8);
sesame_type_value_impl!(u16);
sesame_type_value_impl!(u32);
sesame_type_value_impl!(u64);
sesame_type_value_impl!(bool);
sesame_type_value_impl!(f64);
sesame_type_value_impl!(String);

// Implement SesameType for BBox<T, P>
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: Any, P: Policy + Clone + 'static> SesameTypeDyn<dyn Any> for BBox<T, P> {
    type Out = T;
    fn to_enum(self) -> SesameTypeEnumDyn<dyn Any> {
        SesameTypeEnumDyn::BBox(self.into_any())
    }
    fn from_enum(e: SesameTypeEnumDyn<dyn Any>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Value(v) => match v.downcast() {
                Ok(v) => Ok(*v),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: AnySerialize, P: Policy + Clone + 'static> SesameTypeDyn<dyn AnySerialize> for BBox<T, P> {
    type Out = T;
    fn to_enum(self) -> SesameTypeEnumDyn<dyn AnySerialize> {
        let (t, p) = self.consume();
        SesameTypeEnumDyn::BBox(BBox::new(Box::new(t), p.into_any()))
    }
    fn from_enum(e: SesameTypeEnumDyn<dyn AnySerialize>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Value(v) => match v.upcast_any_box().downcast() {
                Ok(v) => Ok(*v),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}

// Implement SesameType for Option
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: ?Sized, T: SesameTypeDyn<A>> SesameTypeDyn<A> for Option<T> {
    type Out = Option<T::Out>;
    fn to_enum(self) -> SesameTypeEnumDyn<A> {
        match self {
            None => SesameTypeEnumDyn::Vec(Vec::new()),
            Some(t) => SesameTypeEnumDyn::Vec(vec![t.to_enum()]),
        }
    }
    fn from_enum(e: SesameTypeEnumDyn<A>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Vec(mut vec) => match vec.pop() {
                None => Ok(None),
                Some(v) => Ok(Some(T::from_enum(v)?)),
            },
            _ => Err(()),
        }
    }
}

// Implement SesameType for Vec.
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: ?Sized, S: SesameTypeDyn<A>> SesameTypeDyn<A> for Vec<S> {
    type Out = Vec<S::Out>;
    fn to_enum(self) -> SesameTypeEnumDyn<A> {
        SesameTypeEnumDyn::Vec(self.into_iter().map(|s| s.to_enum()).collect())
    }
    fn from_enum(e: SesameTypeEnumDyn<A>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Vec(v) => {
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

// Implement SesameType for HashMap
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: ?Sized, K: ToString + FromStr + Hash + Eq, S: SesameTypeDyn<A>> SesameTypeDyn<A> for HashMap<K, S> {
    type Out = HashMap<K, S::Out>;
    fn to_enum(self) -> SesameTypeEnumDyn<A> {
        SesameTypeEnumDyn::Struct(
            self.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_enum()))
                .collect(),
        )
    }
    fn from_enum(e: SesameTypeEnumDyn<A>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Struct(m) => {
                let mut result = HashMap::new();
                for (k, v) in m.into_iter() {
                    match K::from_str(&k) {
                        Ok(k) => {
                            result.insert(k, S::from_enum(v)?);
                        }
                        Err(_) => return Err(()),
                    }
                }
                Ok(result)
            }
            _ => Err(()),
        }
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: ?Sized> SesameTypeDyn<A> for () {
    type Out = ();
    fn to_enum(self) -> SesameTypeEnumDyn<A> {
        SesameTypeEnumDyn::Vec(Vec::new())
    }
    fn from_enum(e: SesameTypeEnumDyn<A>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Vec(v) if v.len() == 0 => Ok(()),
            _ => Err(()),
        }
    }
}

// Implement SesameType for tuples made up of SesameTypes.
macro_rules! sesame_type_tuple_impl {
  ($([$A:tt,$i:tt]),*) => (
    #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
    impl<DYN: ?Sized, $($A: SesameTypeDyn<DYN>,)*> SesameTypeDyn<DYN> for ($($A,)*) {
        type Out = ($($A::Out,)*);
        fn to_enum(self) -> SesameTypeEnumDyn<DYN> {
            #[allow(non_snake_case)]
            let ($($A,)*) = ($(self.$i.to_enum(),)*);
            SesameTypeEnumDyn::Vec(vec![$($A,)*])
        }
        fn from_enum(e: SesameTypeEnumDyn<DYN>) -> Result<Self::Out, ()> {
            match e {
                SesameTypeEnumDyn::Vec(v) => {
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

sesame_type_tuple_impl!([A, 0]);
sesame_type_tuple_impl!([A, 0], [B, 1]);
sesame_type_tuple_impl!([A, 0], [B, 1], [C, 2]);
sesame_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3]);
sesame_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4]);
sesame_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4], [F, 5]);
sesame_type_tuple_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4], [F, 5], [G, 6]);
sesame_type_tuple_impl!(
    [A, 0],
    [B, 1],
    [C, 2],
    [D, 3],
    [E, 4],
    [F, 5],
    [G, 6],
    [H, 7]
);
sesame_type_tuple_impl!(
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
sesame_type_tuple_impl!(
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
);
sesame_type_tuple_impl!(
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
sesame_type_tuple_impl!(
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

// Implement SesameType for Mutex<T>
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: ?Sized, T: SesameTypeDyn<A>> SesameTypeDyn<A> for Mutex<T> {
    type Out = Mutex<T::Out>;
    fn to_enum(self) -> SesameTypeEnumDyn<A> {
        let t = self.into_inner().unwrap();
        SesameTypeEnumDyn::Vec(vec![t.to_enum()])
    }
    fn from_enum(e: SesameTypeEnumDyn<A>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Vec(mut v) => {
                let t = v.pop().unwrap();
                Ok(Mutex::new(T::from_enum(t)?))
            }
            _ => Err(()),
        }
    }
}

// Implement SesameType for Arc<T>
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: ?Sized, T: SesameTypeDyn<A>> SesameTypeDyn<A> for Arc<T> {
    type Out = Arc<T::Out>;
    fn to_enum(self) -> SesameTypeEnumDyn<A> {
        let t = Arc::into_inner(self).unwrap();
        SesameTypeEnumDyn::Vec(vec![t.to_enum()])
    }
    fn from_enum(e: SesameTypeEnumDyn<A>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnumDyn::Vec(mut v) => {
                let t = v.pop().unwrap();
                Ok(Arc::new(T::from_enum(t)?))
            }
            _ => Err(()),
        }
    }
}