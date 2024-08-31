use std::any::Any;

use crate::AlohomoraType;
use crate::context::{Context, ContextData};
use crate::fold::fold;
use crate::pcr::PrivacyCriticalRegion;
use crate::policy::CloneableAny;
use crate::unbox::UnboxError::{FoldError, PolicyViolation};

#[derive(Debug)]
pub enum UnboxError {
    FoldError,
    PolicyViolation,
}

pub fn unbox<S: AlohomoraType, D: ContextData, C: Clone + AlohomoraType, O, F: FnOnce(S::Out, C::Out) -> O>(
    data: S,
    context: Context<D>,
    functor: PrivacyCriticalRegion<F>,
    arg: C
) -> Result<O, UnboxError> where C::Out: CloneableAny + Clone {
    match fold(data) {
        Ok(data) => {
            match data.into_unbox(context, functor, arg) {
                Err(_) => Err(PolicyViolation),
                Ok(result) => Ok(result),
            }
        },
        _ => Err(FoldError),
    }
}
