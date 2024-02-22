use crate::AlohomoraType;
use crate::context::Context;
use crate::fold::fold;
use crate::pcr::PrivacyCriticalRegion;
use crate::unbox::UnboxError::{FoldError, PolicyViolation};

#[derive(Debug)]
pub enum UnboxError {
    FoldError,
    PolicyViolation,
}

pub fn unbox<S: AlohomoraType, U, D, C, O, F: FnOnce(S::Out, C) -> O>(
        data: S,
        context: &Context<U, D>,
        functor: PrivacyCriticalRegion<F>,
        arg: C) -> Result<O, UnboxError> {
    match fold(data) {
        Err(_) => Err(FoldError),
        Ok(data) => {
            match data.into_unbox(context, functor, arg) {
                Err(_) => Err(PolicyViolation),
                Ok(result) => Ok(result),
            }
        }
    }
}