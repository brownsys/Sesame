use crate::AlohomoraType;
use crate::context::{Context, ContextData};
use crate::fold::fold;
use crate::pcr::PrivacyCriticalRegion;
use crate::unbox::UnboxError::{FoldError, PolicyViolation};

#[derive(Debug)]
pub enum UnboxError {
    FoldError,
    PolicyViolation,
}

pub fn unbox_<S: AlohomoraType, D: ContextData, C, O, F: FnOnce(S::Out, C) -> O>(
    data: S,
    context: Context<D>,
    functor: PrivacyCriticalRegion<F>,
    arg: C
) -> Result<O, UnboxError> {
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