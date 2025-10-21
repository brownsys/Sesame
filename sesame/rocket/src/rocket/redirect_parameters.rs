use sesame::bbox::{BBox, EitherBBox};
use sesame::context::{Context, ContextData};
use sesame::policy::{Policy, Reason, RefPolicy};

use sesame::error::SesameResult;
use sesame::extensions::{
    ExtensionContext, SesameExtension, SesameRefExtension, UncheckedSesameExtension,
};
use std::string::ToString;

// Lightweight: reference to both data and policy.
type RefEitherParam<'a> = EitherBBox<&'a dyn ToString, RefPolicy<'a, dyn Policy + 'a>>;

// Our params may be boxed or clear.
pub trait RedirectParam<'a> {
    fn get(self) -> RefEitherParam<'a>;
}

impl<'a, T: ToString + 'a, P: Policy> RedirectParam<'a> for &'a BBox<T, P> {
    fn get(self) -> RefEitherParam<'a> {
        struct Converter {}
        impl UncheckedSesameExtension for Converter {}
        impl<'a, T: ToString + 'a, P: Policy> SesameRefExtension<'a, T, P, RefEitherParam<'a>>
            for Converter
        {
            fn apply_ref(&mut self, data: &'a T, policy: &'a P) -> RefEitherParam<'a> {
                EitherBBox::Right(BBox::new(data, RefPolicy::new(policy)))
            }
        }
        self.unchecked_extension_ref(&mut Converter {})
    }
}

impl<'a, T: ToString + 'a, P: Policy> RedirectParam<'a> for &'a EitherBBox<T, P> {
    fn get(self) -> RefEitherParam<'a> {
        match self {
            EitherBBox::Left(t) => EitherBBox::Left(t),
            EitherBBox::Right(bbox) => bbox.get(),
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
    fn into<D: ContextData>(self, url: &str, context: Context<D>) -> SesameResult<RedirectParams>;
}

// Can make Params from empty tuple.
impl IntoRedirectParams for () {
    fn into<D: ContextData>(
        self,
        _url: &str,
        _context: Context<D>,
    ) -> SesameResult<RedirectParams> {
        Ok(RedirectParams {
            parameters: Vec::new(),
        })
    }
}

// Check policy before adding parameter to redirect string.
struct RedirectPolicyCheck {
    params: Vec<String>,
}
impl RedirectPolicyCheck {
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }
    pub fn push(&mut self, v: String) {
        self.params.push(v);
    }
    pub fn into_redirect_params(self) -> RedirectParams {
        RedirectParams {
            parameters: self.params,
        }
    }
}
impl<'a> SesameExtension<&'a dyn ToString, RefPolicy<'a, dyn Policy + 'a>, ()>
    for RedirectPolicyCheck
{
    fn apply(&mut self, data: &'a dyn ToString, _policy: RefPolicy<'a, dyn Policy + 'a>) -> () {
        self.params.push(data.to_string());
    }
}

// Can make params from inlined tuples.
macro_rules! into_params_impl {
  ($([$A:ident,$a:ident,$l:lifetime]),*) => (
    impl<$($l,)* $($A: RedirectParam<$l>,)*> IntoRedirectParams for ($($A,)*) {
      fn into<DD : ContextData>(self, url: &str, context: Context<DD>) -> SesameResult<RedirectParams> {
        let ($($a,)*) = self;
        let context = ExtensionContext::new(context);
        let mut ext = RedirectPolicyCheck::new();

        $(match $a.get() {
            EitherBBox::Left(v) => ext.push(v.to_string()),
            EitherBBox::Right(b) => {
                b.checked_extension(&mut ext, &context, Reason::Redirect(url))?;
            },
        };)*

        Ok(ext.into_redirect_params())
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
