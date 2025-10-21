// BBox
use sesame::bbox::EitherBBox;
use sesame::context::{Context, ContextData};
use sesame::error::SesameError;
use sesame::extensions::{
    ExtensionContext, SesameExtension, SesameRefExtension, UncheckedSesameExtension,
};
use sesame::policy::{AnyPolicy, Reason};

use crate::BBoxParam;

// Use Sesame Extension to execute policy check on BBoxed parameters
// and retrieve the data when policy check is successful for writing to the DB.
struct PolicyCheck {
    vec: Vec<mysql::Value>,
}
impl PolicyCheck {
    pub fn new() -> PolicyCheck {
        PolicyCheck { vec: Vec::new() }
    }
    pub fn push(&mut self, value: mysql::Value) {
        self.vec.push(value);
    }
    pub fn into_params(self) -> mysql::params::Params {
        mysql::params::Params::Positional(self.vec)
    }
}
impl SesameExtension<mysql::Value, AnyPolicy, ()> for PolicyCheck {
    fn apply(&mut self, data: mysql::Value, _policy: AnyPolicy) {
        self.vec.push(data)
    }
}

// Our params could be mixed boxed and clear.
pub enum BBoxParams {
    Empty,
    // Named(HashMap<String, Value>),
    Positional(Vec<EitherBBox<mysql::Value, AnyPolicy>>),
}

// private helper function.
impl BBoxParams {
    pub(super) fn transform<D: ContextData>(
        self,
        context: Context<D>,
        reason: Reason,
    ) -> Result<mysql::params::Params, SesameError> {
        match self {
            BBoxParams::Empty => Ok(mysql::params::Params::Empty),
            BBoxParams::Positional(vec) => {
                let context = ExtensionContext::new(context);
                let mut ext = PolicyCheck::new();
                for v in vec.into_iter() {
                    match v {
                        EitherBBox::Left(v) => ext.push(v),
                        EitherBBox::Right(bbox) => {
                            bbox.checked_extension(&mut ext, &context, reason.clone())?;
                        }
                    }
                }
                Ok(ext.into_params())
            }
        }
    }

    pub(super) fn to_reason(&self) -> Vec<mysql::Value> {
        match self {
            BBoxParams::Empty => Vec::new(),
            BBoxParams::Positional(v) => {
                struct Converter {}
                impl UncheckedSesameExtension for Converter {}
                impl<'a> SesameRefExtension<'a, mysql::Value, AnyPolicy, mysql::Value> for Converter {
                    fn apply_ref(
                        &mut self,
                        data: &'a mysql::Value,
                        _policy: &'a AnyPolicy,
                    ) -> mysql::Value {
                        data.clone()
                    }
                }
                v.into_iter()
                    .map(|either| match either {
                        EitherBBox::Left(value) => value.clone(),
                        EitherBBox::Right(bbox) => bbox.unchecked_extension_ref(&mut Converter {}),
                    })
                    .collect()
            }
        }
    }
}

// Can make Params from empty and Vec.
impl From<()> for BBoxParams {
    fn from(_: ()) -> BBoxParams {
        BBoxParams::Empty
    }
}
impl<T: BBoxParam> From<Vec<T>> for BBoxParams {
    fn from(x: Vec<T>) -> BBoxParams {
        if x.is_empty() {
            BBoxParams::Empty
        } else {
            BBoxParams::Positional(x.into_iter().map(|v| v.get()).collect())
        }
    }
}

// Can make params from inlined function arguments.
macro_rules! into_params_impl {
  ($([$A:ident,$a:ident]),*) => (
    impl<$($A: BBoxParam,)*> From<($($A,)*)> for BBoxParams {
      fn from(x: ($($A,)*)) -> BBoxParams {
        let ($($a,)*) = x;
        BBoxParams::Positional(vec![
          $($a.get(),)*
        ])
      }
    }
  );
}
into_params_impl!([A, a]);
into_params_impl!([A, a], [B, b]);
into_params_impl!([A, a], [B, b], [C, c]);
into_params_impl!([A, a], [B, b], [C, c], [D, d]);
into_params_impl!([A, a], [B, b], [C, c], [D, d], [E, e]);
into_params_impl!([A, a], [B, b], [C, c], [D, d], [E, e], [F, f]);
into_params_impl!([A, a], [B, b], [C, c], [D, d], [E, e], [F, f], [G, g]);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h]
);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h],
    [I, i]
);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h],
    [I, i],
    [J, j]
);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h],
    [I, i],
    [J, j],
    [K, k]
);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h],
    [I, i],
    [J, j],
    [K, k],
    [L, l]
);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h],
    [I, i],
    [J, j],
    [K, k],
    [L, l],
    [M, m]
);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h],
    [I, i],
    [J, j],
    [K, k],
    [L, l],
    [M, m],
    [N, n]
);

#[cfg(test)]
mod tests {
    use crate::BBoxParams;
    use mysql::prelude::{FromValue, ToValue};
    use mysql::Params;
    use sesame::bbox::{BBox, EitherBBox};
    use sesame::context::Context;
    use sesame::policy::{AnyPolicy, NoPolicy, Reason};
    use std::boxed::Box;

    fn helper1<T: FromValue + Eq>(b: &BBox<mysql::Value, AnyPolicy>, t: T) -> bool {
        let v = b
            .as_ref()
            .specialize_policy_ref::<NoPolicy>()
            .unwrap()
            .discard_box()
            .to_value();
        mysql::from_value::<T>(v) == t
    }
    fn helper2<T: FromValue + Eq>(b: &mysql::Value, t: T) -> bool {
        mysql::from_value::<T>(b.clone()) == t
    }

    #[test]
    fn make_params_from_mixed_tuple() {
        let b1 = BBox::new(String::from("kinan"), NoPolicy {});
        let b2 = BBox::new(10, NoPolicy {});
        let b3 = 100;
        let b4 = String::from("test");
        let params = BBoxParams::from((b1, b2, b3, b4));

        // Test construction.
        assert!(matches!(&params, BBoxParams::Positional(v) if v.len() == 4));
        if let BBoxParams::Positional(vec) = &params {
            assert!(matches!(&vec[0], EitherBBox::Right(b) if helper1(&b, String::from("kinan"))));
            assert!(matches!(&vec[1], EitherBBox::Right(b) if helper1(&b, 10i32)));
            assert!(matches!(&vec[2], EitherBBox::Left(b) if helper2(&b, 100i32)));
            assert!(matches!(&vec[3], EitherBBox::Left(b) if helper2(&b, String::from("test"))));
        }

        // Test unboxing.
        let params = params.transform(Context::test(()), Reason::Custom(&Box::new(())));
        assert!(matches!(&params, Ok(Params::Positional(v)) if v.len() == 4));
        if let Ok(Params::Positional(vec)) = &params {
            assert_eq!(mysql::from_value::<String>(vec[0].clone()), "kinan");
            assert_eq!(mysql::from_value::<i32>(vec[1].clone()), 10i32);
            assert_eq!(mysql::from_value::<i32>(vec[2].clone()), 100i32);
            assert_eq!(mysql::from_value::<String>(vec[3].clone()), "test");
        }
    }
}
