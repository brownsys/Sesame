use crate::fold::fold;
use crate::pcon::PCon;
use crate::policy::{AnyPolicy, PolicyDyn};
use crate::SesameType;

use std::any::Any;

#[derive(Clone, Copy)]
pub struct VerifiedRegion<F> {
    f: F,
}
impl<F> VerifiedRegion<F> {
    pub const fn new(f: F) -> Self {
        VerifiedRegion { f }
    }
    pub fn get_functor(self) -> F {
        self.f
    }
}

// Executes a PCR over some boxed type.
pub fn execute_verified<
    PDyn: PolicyDyn + ?Sized,
    S: SesameType<dyn Any, PDyn>,
    O,
    F: Fn(S::Out) -> O,
>(
    data: S,
    functor: VerifiedRegion<F>,
) -> Result<PCon<O, AnyPolicy<PDyn>>, ()> {
    let data = fold(data)?;
    let (t, p) = data.consume();
    let functor = functor.get_functor();
    Ok(PCon::new(functor(t), p))
}
