// BBox
use crate::bbox::EitherBBox;
use crate::db::BBoxParam;
use crate::policy::AnyPolicy;

// Our params could be mixed boxed and clear.
#[derive(Clone)]
pub enum BBoxParams {
    Empty,
    // Named(HashMap<String, Value>),
    Positional(Vec<EitherBBox<mysql::Value, AnyPolicy>>),
}

// private helper function.
impl BBoxParams {
    pub(super) fn transform(self) -> mysql::params::Params {
        match self {
            BBoxParams::Empty => mysql::params::Params::Empty,
            BBoxParams::Positional(vec) => {
                let unboxed = vec
                    .into_iter()
                    .map(|v| match v {
                        EitherBBox::Value(v) => v,
                        EitherBBox::BBox(bbox) => bbox.consume().0,
                    })
                    .collect();
                mysql::params::Params::Positional(unboxed)
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
);into_params_impl!(
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
    use mysql::Params;
    use mysql::prelude::FromValue;
    use crate::bbox::{BBox, EitherBBox};
    use crate::db::{BBoxParam, BBoxParams};
    use crate::policy::{AnyPolicy, NoPolicy};

    fn helper1<T: FromValue + Eq>(b: &BBox<mysql::Value, AnyPolicy>, t: T) -> bool {
        mysql::from_value::<T>(b.data().clone()) == t
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
            assert!(matches!(vec[0].clone().get(), EitherBBox::BBox(b) if helper1(&b, String::from("kinan"))));
            assert!(matches!(vec[1].clone().get(), EitherBBox::BBox(b) if helper1(&b, 10i32)));
            assert!(matches!(vec[2].clone().get(), EitherBBox::Value(b) if helper2(&b, 100i32)));
            assert!(matches!(vec[3].clone().get(), EitherBBox::Value(b) if helper2(&b, String::from("test"))));
        }

        // Test unboxing.
        let params = params.transform();
        assert!(matches!(&params, Params::Positional(v) if v.len() == 4));
        if let Params::Positional(vec) = &params {
            assert_eq!(mysql::from_value::<String>(vec[0].clone()), "kinan");
            assert_eq!(mysql::from_value::<i32>(vec[1].clone()), 10i32);
            assert_eq!(mysql::from_value::<i32>(vec[2].clone()), 100i32);
            assert_eq!(mysql::from_value::<String>(vec[3].clone()), "test");
        }
    }
}
