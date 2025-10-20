use sesame::bbox::{BBox, EitherBBox};
use sesame::context::{Context, ContextData, UnprotectedContext};
use sesame::policy::{Policy, Reason, RefPolicy};

use std::string::ToString;

// Lightweight: reference to both data and policy.
type RefEitherParam<'a> = EitherBBox<&'a dyn ToString, RefPolicy<'a, dyn Policy + 'a>>;

// Our params may be boxed or clear.
pub trait RedirectParam<'a> {
    fn get(self) -> RefEitherParam<'a>;
}

impl<'a, T: ToString + 'a, P: Policy> RedirectParam<'a> for &'a BBox<T, P> {
    fn get(self) -> RefEitherParam<'a> {
        EitherBBox::Right(BBox::new(self.data(), RefPolicy::new(self.policy())))
    }
}

impl<'a, T: ToString + 'a, P: Policy> RedirectParam<'a> for &'a EitherBBox<T, P> {
    fn get(self) -> RefEitherParam<'a> {
        match self {
            EitherBBox::Left(t) => EitherBBox::Left(t),
            EitherBBox::Right(bbox) => {
                EitherBBox::Right(BBox::new(bbox.data(), RefPolicy::new(bbox.policy())))
            }
        }
    }
}

// Implement for basic types.
macro_rules! redirect_param_impl {
  ($($T:ty,)+) => (
    $(
    impl<'a> RedirectParam<'a> for &'a $T {
        fn get(self) -> RefEitherParam<'a> {
            EitherBBox::Left(self)
        }
    }
    )+
  );
}
redirect_param_impl!(String, &str,);
redirect_param_impl!(u8, u16, u32, u64, u128, usize,);
redirect_param_impl!(i8, i16, i32, i64, i128, isize,);
redirect_param_impl!(bool, char, f32, f64,);

// Parameters.
pub struct RedirectParams {
    // Private: client code cannot see these.
    pub(super) parameters: Vec<String>,
}

pub trait IntoRedirectParams {
    fn into<D: ContextData>(self, url: &str, context: Context<D>) -> RedirectParams;
}

// Can make Params from empty tuple.
impl IntoRedirectParams for () {
    fn into<D: ContextData>(self, _url: &str, _context: Context<D>) -> RedirectParams {
        RedirectParams {
            parameters: Vec::new(),
        }
    }
}

// Can make params from inlined tuples.
macro_rules! into_params_impl {
  ($([$A:ident,$a:ident,$l:lifetime]),*) => (
    impl<$($l,)* $($A: RedirectParam<$l>,)*> IntoRedirectParams for ($($A,)*) {
      fn into<DD : ContextData>(self, url: &str, context: Context<DD>) -> RedirectParams {
        let ($($a,)*) = self;
        let context = UnprotectedContext::from(context);

        $(let $a = match $a.get() {
            EitherBBox::Left(v) => v.to_string(),
            EitherBBox::Right(b) => {
                if b.policy().check(&context, Reason::Redirect(url)) {
                    b.data().to_string()
                } else {
                    panic!("failed policy check");
                }
            },
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
