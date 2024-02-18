// BBox
use crate::bbox::EitherBBox;
use crate::db::BBoxParam;

// Our params could be mixed boxed and clear.
pub enum BBoxParams {
    Empty,
    // Named(HashMap<String, Value>),
    Positional(Vec<BBoxParam>),
}

// private helper function.
pub(super) fn unbox_params(params: BBoxParams) -> mysql::params::Params {
    match params {
        BBoxParams::Empty => mysql::params::Params::Empty,
        BBoxParams::Positional(vec) => {
            let unboxed = vec
                .into_iter()
                .map(|v: BBoxParam| match v {
                    BBoxParam(EitherBBox::Value(v)) => v,
                    BBoxParam(EitherBBox::BBox(bbox)) => bbox.t,
                })
                .collect();
            mysql::params::Params::Positional(unboxed)
        }
    }
}

// Can make Params from empty and Vec.
impl From<()> for BBoxParams {
    fn from(_: ()) -> BBoxParams {
        BBoxParams::Empty
    }
}
impl<T: Into<BBoxParam>> From<Vec<T>> for BBoxParams {
    fn from(x: Vec<T>) -> BBoxParams {
        let mut raw_params: Vec<BBoxParam> = Vec::new();
        for v in x.into_iter() {
            raw_params.push(v.into());
        }
        if raw_params.is_empty() {
            BBoxParams::Empty
        } else {
            BBoxParams::Positional(raw_params)
        }
    }
}

// Can make params from inlined function arguments.
macro_rules! into_params_impl {
  ($([$A:ident,$a:ident]),*) => (
    impl<$($A: Into<BBoxParam>,)*> From<($($A,)*)> for BBoxParams {
      fn from(x: ($($A,)*)) -> BBoxParams {
        let ($($a,)*) = x;
        BBoxParams::Positional(vec![
          $($a.into(),)*
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