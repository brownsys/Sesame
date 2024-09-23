use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::bbox::{BBox};
use crate::policy::{AnyPolicy, NoPolicy, OptionPolicy, Policy};

// 0-sized type that can only be constructed by Alohomora.
pub struct Unwrapper {
    _phantom: (),
}
impl Unwrapper {
    pub(crate) fn new() -> Self {
        Unwrapper { _phantom: () }
    }
    pub fn unwrap<T, P: Policy>(&self, bbox: BBox<T, P>) -> (T, P) {
        bbox.consume()
    }
}

// Public: client code should derive this for structs that they want to unbox, fold, or pass to
// sandboxes.
// TODO(babman): Do not use AnyPolicy anywhere here.
// TODO(babman): Propogate OptionPolicy upwards with join (see test_nested_boxes in alohomora_derive).
// TODO(babman): Update derive macro to be smarter about output policy type.
pub trait AlohomoraType {
    type Out;     // Unboxed form of struct
    type Policy: Policy + Clone + 'static;  // Policy
    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()>;
}

// Implement AlohomoraType for various primitives.
macro_rules! alohomora_type_impl {
    ($T: ty) => {
        #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl AlohomoraType for $T {
            type Out = $T;
            type Policy = NoPolicy;
            fn inner_fold(self, _unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
                Ok((self, NoPolicy {}))
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
    type Policy = OptionPolicy<T::Policy>;
    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        match self {
            None => Ok((None, OptionPolicy::NoPolicy)),
            Some(t) => {
                let (t, p) = t.inner_fold(unwrapper)?;
                Ok((Some(t), OptionPolicy::Policy(p)))
            },
        }
    }
}

// Implement AlohomoraType for BBox<T, P>
#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T, P: Policy + Clone + 'static> AlohomoraType for BBox<T, P> {
    type Out = T;
    type Policy = P;
    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        Ok(unwrapper.unwrap(self))
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<S: AlohomoraType> AlohomoraType for Vec<S> {
    type Out = Vec<S::Out>;
    type Policy = OptionPolicy<S::Policy>;
    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        let mut v = Vec::with_capacity(self.len());

        // Get first element.
        let mut it = self.into_iter();
        let mut p = match it.next() {
            None => { return Ok((v, OptionPolicy::NoPolicy)); },
            Some(e) => {
                let (t, p) = e.inner_fold(unwrapper)?;
                v.push(t);
                p
            },
        };

        // Iterate over remaining elements.
        for e in it {
            let (t, p2) = e.inner_fold(unwrapper)?;
            v.push(t);
            p = p.join_logic(p2)?;
        }

        Ok((v, OptionPolicy::Policy(p)))
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<K: Hash + Eq, S: AlohomoraType> AlohomoraType for HashMap<K, S> {
    type Out = HashMap<K, S::Out>;
    type Policy = OptionPolicy<S::Policy>;
    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        let mut v = HashMap::with_capacity(self.len());

        // Get first element.
        let mut it = self.into_iter();
        let mut p = match it.next() {
            None => { return Ok((v, OptionPolicy::NoPolicy)); },
            Some((k, e)) => {
                let (t, p) = e.inner_fold(unwrapper)?;
                v.insert(k, t);
                p
            },
        };

        // Iterate over remaining elements.
        for (k, e) in it {
            let (t, p2) = e.inner_fold(unwrapper)?;
            v.insert(k, t);
            p = p.join_logic(p2)?;
        }

        Ok((v, OptionPolicy::Policy(p)))
    }
}


#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl AlohomoraType for () {
    type Out = ();
    type Policy = NoPolicy;
    fn inner_fold(self, _unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        Ok(((), NoPolicy {}))
    }
}

// Implement AlohomoraType for tuples made up of AlohomoraTypes.
macro_rules! alohomora_type_tuple_impl {
  ($([$A:tt,$i:tt]),*) => (
    #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
    impl<$($A: AlohomoraType,)*> AlohomoraType for ($($A,)*) {
        type Out = ($($A::Out,)*);
        type Policy = AnyPolicy;

        fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
            #[allow(non_snake_case)]
            let ($($A,)*) = ($(self.$i.inner_fold(unwrapper)?,)*);
            let (data, policies) = (($($A.0,)*), vec![$(AnyPolicy::new($A.1),)*]);
            let policy = policies.into_iter().reduce(|p1, p2| p1.join(p2).unwrap()).unwrap();
            Ok((data, policy))
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
    type Policy = T::Policy;
    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        let t = Arc::into_inner(self).unwrap().into_inner().unwrap();
        let (t, p) = t.inner_fold(unwrapper)?;
        Ok((Arc::new(Mutex::new(t)), p))
    }
}
