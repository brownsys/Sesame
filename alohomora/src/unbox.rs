use crate::context::{Context, ContextData};
use crate::fold::fold;
use crate::pcr::PrivacyCriticalRegion;
use crate::policy::CloneableAny;
use crate::unbox::UnboxError::{FoldError, PolicyViolation};
use crate::SesameType;

#[derive(Debug)]
pub enum UnboxError {
    FoldError,
    PolicyViolation,
}

pub fn unbox<
    S: SesameType,
    D: ContextData,
    C: Clone + SesameType,
    O,
    F: FnOnce(S::Out, C::Out) -> O,
>(
    data: S,
    context: Context<D>,
    functor: PrivacyCriticalRegion<F>,
    arg: C,
) -> Result<O, UnboxError>
where
    C::Out: CloneableAny + Clone,
{
    match fold(data) {
        Ok(data) => match data.into_unbox(context, functor, arg) {
            Err(_) => Err(PolicyViolation),
            Ok(result) => Ok(result),
        },
        _ => Err(FoldError),
    }
}
