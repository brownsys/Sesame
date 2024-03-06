use crate::bbox::{BBox, EitherBBox};
use crate::policy::{Policy, RefPolicy};

use std::string::ToString;

// Lightweight: reference to both data and policy.
type RefEitherParam<'a> = EitherBBox<&'a dyn ToString, RefPolicy<'a, dyn Policy + 'a>>;

// Our params may be boxed or clear.
pub trait RedirectParam<'a> {
    fn get(self) -> RefEitherParam<'a>;
}

impl<'a, T: ToString + 'a, P: Policy> RedirectParam<'a> for &'a BBox<T, P> {
    fn get(self) -> RefEitherParam<'a> {
        EitherBBox::BBox(BBox::new(self.data(), RefPolicy::new(self.policy())))
    }
}

impl<'a, T: ToString + 'a> RedirectParam<'a> for &'a T {
    fn get(self) -> RefEitherParam<'a> {
        EitherBBox::Value(self)
    }
}

impl<'a, T: ToString + 'a, P: Policy> RedirectParam<'a> for &'a EitherBBox<T, P> {
    fn get(self) -> RefEitherParam<'a> {
        match self {
            EitherBBox::Value(t) => EitherBBox::Value(t),
            EitherBBox::BBox(bbox) =>
                EitherBBox::BBox(BBox::new(bbox.data(), RefPolicy::new(bbox.policy()))),
        }
    }
}

// Parameters.
pub struct RedirectParams {
    // Private: client code cannot see these.
    pub(super) parameters: Vec<String>,
}

// Can make Params from empty tuple.
impl From<()> for RedirectParams {
    fn from(_: ()) -> RedirectParams {
        RedirectParams { parameters: Vec::new() }
    }
}

// Can make params from inlined tuples.
macro_rules! into_params_impl {
  ($([$A:ident,$a:ident,$l:lifetime]),*) => (
    impl<$($l,)* $($A: RedirectParam<$l>,)*> From<($($A,)*)> for RedirectParams {
      fn from(x: ($($A,)*)) -> RedirectParams {
        let ($($a,)*) = x;

        $(let $a = match $a.get() {
            EitherBBox::Value(v) => v.to_string(),
            EitherBBox::BBox(b) => b.data().to_string(),
        };)*

        RedirectParams {
            parameters: vec![$($a,)*]
        }
      }
    }
  );
}
into_params_impl!([A, a, 'a]);
into_params_impl!([A, a, 'a], [B, b, 'b]);
into_params_impl!([A, a, 'a], [B, b, 'b], [C, c, 'c]);
into_params_impl!([A, a, 'a], [B, b, 'b], [C, c, 'c], [D, d, 'd]);
into_params_impl!([A, a, 'a], [B, b, 'b], [C, c, 'c], [D, d, 'd], [E, e, 'e]);
into_params_impl!([A, a, 'a], [B, b, 'b], [C, c, 'c], [D, d, 'd], [E, e, 'e], [F, f, 'f]);
into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g]);
into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g],
    [H, h, 'h]
);
into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g],
    [H, h, 'h],
    [I, i, 'i]
);
into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g],
    [H, h, 'h],
    [I, i, 'i],
    [J, j, 'j]
);into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g],
    [H, h, 'h],
    [I, i, 'i],
    [J, j, 'j],
    [K, k, 'k]
);
into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g],
    [H, h, 'h],
    [I, i, 'i],
    [J, j, 'j],
    [K, k, 'k],
    [L, l, 'l]
);
into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g],
    [H, h, 'h],
    [I, i, 'i],
    [J, j, 'j],
    [K, k, 'k],
    [L, l, 'l],
    [M, m, 'm]
);
into_params_impl!(
    [A, a, 'a],
    [B, b, 'b],
    [C, c, 'c],
    [D, d, 'd],
    [E, e, 'e],
    [F, f, 'f],
    [G, g, 'g],
    [H, h, 'h],
    [I, i, 'i],
    [J, j, 'j],
    [K, k, 'k],
    [L, l, 'l],
    [M, m, 'm],
    [N, n, 'n]
);