use itertools::Itertools;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::bbox::BBox;
use crate::policy::{AnyPolicy, AnyPolicyable, PolicyDyn, PolicyDynRelation};
use crate::sesame_type::r#enum::SesameTypeEnum;
use crate::sesame_type::r#type::{SesameType, SesameTypeOut};
use crate::sesame_type_dyns::{SesameDyn, SesameDynRelation};

// Implement SesameType for various primitives.
macro_rules! sesame_type_dyn_primitives_impl {
    ($T: ty) => {
        #[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
        impl SesameTypeOut for $T {
            type Out = $T;
        }

        #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl<DT: SesameDyn + ?Sized, P: PolicyDyn + ?Sized> SesameType<DT, P> for $T
        where
            DT: SesameDynRelation<$T>,
        {
            fn to_enum(self) -> SesameTypeEnum<DT, P> {
                SesameTypeEnum::Value(DT::boxed_dyn(self))
            }
            fn from_enum(e: SesameTypeEnum<DT, P>) -> Result<Self::Out, ()> {
                match e {
                    SesameTypeEnum::Value(v) => match v.upcast_box().downcast() {
                        Err(_) => Err(()),
                        Ok(v) => Ok(*v),
                    },
                    _ => Err(()),
                }
            }
        }
    };
}

sesame_type_dyn_primitives_impl!(i8);
sesame_type_dyn_primitives_impl!(i16);
sesame_type_dyn_primitives_impl!(i32);
sesame_type_dyn_primitives_impl!(i64);
sesame_type_dyn_primitives_impl!(u8);
sesame_type_dyn_primitives_impl!(u16);
sesame_type_dyn_primitives_impl!(u32);
sesame_type_dyn_primitives_impl!(u64);
sesame_type_dyn_primitives_impl!(bool);
sesame_type_dyn_primitives_impl!(f64);
sesame_type_dyn_primitives_impl!(String);

// Implement SesameType for BBox<T, P>
#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl<T: Any, P: AnyPolicyable> SesameTypeOut for BBox<T, P> {
    type Out = T;
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<
        T: Any,
        DT: SesameDyn + ?Sized + SesameDynRelation<T> + Any,
        P: AnyPolicyable,
        PT: PolicyDyn + ?Sized + PolicyDynRelation<P>,
    > SesameType<DT, PT> for BBox<T, P>
{
    fn to_enum(self) -> SesameTypeEnum<DT, PT> {
        let (t, p) = self.consume();
        SesameTypeEnum::BBox(BBox::new(DT::boxed_dyn(t), AnyPolicy::new(p)))
    }
    fn from_enum(e: SesameTypeEnum<DT, PT>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Value(v) => match v.upcast_box().downcast() {
                Ok(v) => Ok(*v),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}

// Implement SesameType for Option
#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl<T: SesameTypeOut> SesameTypeOut for Option<T> {
    type Out = Option<T::Out>;
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: SesameDyn + ?Sized, P: PolicyDyn + ?Sized, T: SesameType<A, P>> SesameType<A, P>
    for Option<T>
{
    fn to_enum(self) -> SesameTypeEnum<A, P> {
        match self {
            None => SesameTypeEnum::Vec(Vec::new()),
            Some(t) => SesameTypeEnum::Vec(vec![t.to_enum()]),
        }
    }
    fn from_enum(e: SesameTypeEnum<A, P>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Vec(mut vec) => match vec.pop() {
                None => Ok(None),
                Some(v) => Ok(Some(T::from_enum(v)?)),
            },
            _ => Err(()),
        }
    }
}

// Implement SesameType for Vec.
#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl<S: SesameTypeOut> SesameTypeOut for Vec<S> {
    type Out = Vec<S::Out>;
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: SesameDyn + ?Sized, P: PolicyDyn + ?Sized, S: SesameType<A, P>> SesameType<A, P>
    for Vec<S>
{
    fn to_enum(self) -> SesameTypeEnum<A, P> {
        SesameTypeEnum::Vec(self.into_iter().map(|s| s.to_enum()).collect())
    }
    fn from_enum(e: SesameTypeEnum<A, P>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Vec(v) => {
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
#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl<K: ToString + FromStr + Hash + Eq, S: SesameTypeOut> SesameTypeOut for HashMap<K, S> {
    type Out = HashMap<K, S::Out>;
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<
        A: SesameDyn + ?Sized,
        P: PolicyDyn + ?Sized,
        K: ToString + FromStr + Hash + Eq,
        S: SesameType<A, P>,
    > SesameType<A, P> for HashMap<K, S>
{
    fn to_enum(self) -> SesameTypeEnum<A, P> {
        SesameTypeEnum::Struct(
            self.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_enum()))
                .collect(),
        )
    }
    fn from_enum(e: SesameTypeEnum<A, P>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Struct(m) => {
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


// ()
#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl SesameTypeOut for () {
    type Out = ();
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: SesameDyn + ?Sized, P: PolicyDyn + ?Sized> SesameType<A, P> for ()
where
    A: SesameDynRelation<()>,
{
    fn to_enum(self) -> SesameTypeEnum<A, P> {
        SesameTypeEnum::Vec(Vec::new())
    }
    fn from_enum(e: SesameTypeEnum<A, P>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Vec(v) if v.len() == 0 => Ok(()),
            _ => Err(()),
        }
    }
}

// Implement SesameType for tuples made up of SesameTypes.
macro_rules! sesame_type_dyn_tuples_impl {
  ($([$A:tt,$i:tt]),*) => (
    #[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
    impl<$($A: SesameTypeOut,)*> SesameTypeOut for ($($A,)*) {
        type Out = ($($A::Out,)*);
    }

    #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
    impl<DYN: SesameDyn + ?Sized, PDYN: PolicyDyn + ?Sized, $($A: SesameType<DYN, PDYN>,)*> SesameType<DYN, PDYN> for ($($A,)*) {
        fn to_enum(self) -> SesameTypeEnum<DYN, PDYN> {
            #[allow(non_snake_case)]
            let ($($A,)*) = ($(self.$i.to_enum(),)*);
            SesameTypeEnum::Vec(vec![$($A,)*])
        }
        fn from_enum(e: SesameTypeEnum<DYN, PDYN>) -> Result<Self::Out, ()> {
            match e {
                SesameTypeEnum::Vec(v) => {
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

sesame_type_dyn_tuples_impl!([A, 0]);
sesame_type_dyn_tuples_impl!([A, 0], [B, 1]);
sesame_type_dyn_tuples_impl!([A, 0], [B, 1], [C, 2]);
sesame_type_dyn_tuples_impl!([A, 0], [B, 1], [C, 2], [D, 3]);
sesame_type_dyn_tuples_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4]);
sesame_type_dyn_tuples_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4], [F, 5]);
sesame_type_dyn_tuples_impl!([A, 0], [B, 1], [C, 2], [D, 3], [E, 4], [F, 5], [G, 6]);
sesame_type_dyn_tuples_impl!(
    [A, 0],
    [B, 1],
    [C, 2],
    [D, 3],
    [E, 4],
    [F, 5],
    [G, 6],
    [H, 7]
);
sesame_type_dyn_tuples_impl!(
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
sesame_type_dyn_tuples_impl!(
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
sesame_type_dyn_tuples_impl!(
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
sesame_type_dyn_tuples_impl!(
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
#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl<T: SesameTypeOut> SesameTypeOut for Mutex<T> {
    type Out = Mutex<T::Out>;
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: SesameDyn + ?Sized, P: PolicyDyn + ?Sized, T: SesameType<A, P>> SesameType<A, P>
    for Mutex<T>
{
    fn to_enum(self) -> SesameTypeEnum<A, P> {
        let t = self.into_inner().unwrap();
        SesameTypeEnum::Vec(vec![t.to_enum()])
    }
    fn from_enum(e: SesameTypeEnum<A, P>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Vec(mut v) => {
                let t = v.pop().unwrap();
                Ok(Mutex::new(T::from_enum(t)?))
            }
            _ => Err(()),
        }
    }
}

// Implement SesameType for Arc<T>
#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl<T: SesameTypeOut> SesameTypeOut for Arc<T> {
    type Out = Arc<T::Out>;
}
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<A: SesameDyn + ?Sized, P: PolicyDyn + ?Sized, T: SesameType<A, P>> SesameType<A, P>
    for Arc<T>
{
    fn to_enum(self) -> SesameTypeEnum<A, P> {
        let t = Arc::into_inner(self).unwrap();
        SesameTypeEnum::Vec(vec![t.to_enum()])
    }
    fn from_enum(e: SesameTypeEnum<A, P>) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Vec(mut v) => {
                let t = v.pop().unwrap();
                Ok(Arc::new(T::from_enum(t)?))
            }
            _ => Err(()),
        }
    }
}
