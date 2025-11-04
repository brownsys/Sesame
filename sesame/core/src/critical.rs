use std::any::Any;

use crate::context::{Context, ContextData};
use crate::error::SesameError;
use crate::fold::fold;
use crate::policy::{AnyPolicy, Policy, PolicyDyn};
use crate::SesameType;

// Will hold signature of reviewer.
pub struct Signature {
    pub username: &'static str,
    pub signature: &'static str,
}

// A region of this type must be signed.
#[derive(Clone, Copy)]
pub struct CriticalRegion<F> {
    f: F,
}
impl<F> CriticalRegion<F> {
    pub const fn new(f: F, _fn_reviewer: Signature) -> Self {
        CriticalRegion { f }
    }
    pub fn get_functor(self) -> F {
        self.f
    }
}

pub fn execute_critical<S: SesameType, D: ContextData, C: SesameType, O, F: FnOnce(S::Out, C::Out) -> O>(
    data: S,
    context: Context<D>,
    functor: CriticalRegion<F>,
    arg: C,
) -> Result<O, SesameError>
where
    C::Out: Any,
{
    match fold(data) {
        Ok(data) => {
            let name = data.policy().name().to_string();
            match data.into_critical(context, functor, arg) {
                Ok(result) => Ok(result),
                Err(_) => Err(SesameError::PolicyCheckFailed(name)),
            }
        },
        _ => Err(SesameError::SesameTypeFoldFailed(String::from("fold failed"))),
    }
}

// A region of this type must be signed.
#[derive(Clone, Copy)]
pub struct UncheckedCriticalRegion<F> {
    f: F,
}
impl<F> UncheckedCriticalRegion<F> {
    pub const fn new(f: F, _fn_reviewer: Signature) -> Self {
        UncheckedCriticalRegion { f }
    }
    pub fn get_functor(self) -> F {
        self.f
    }
}

// Executes a critical region over some SesameType without a policy check.
// WARNING: THIS IS A LAST RESORT; AKIN TO DECLASSIFY IN IFC.
//          PREFER TO USE execute_critical() UNLESS YOU KNOW WHAT YOU ARE DOING.
pub fn execute_critical_unchecked<
    PDyn: PolicyDyn + ?Sized,
    S: SesameType<dyn Any, PDyn>,
    C,
    O,
    F: FnOnce(S::Out, AnyPolicy<PDyn>, C) -> O,
>(
    data: S,
    functor: UncheckedCriticalRegion<F>,
    arg: C,
) -> Result<O, SesameError> {
    let data = match fold(data) {
        Ok(data) => data,
        _ => { return Err(SesameError::SesameTypeFoldFailed(String::from("fold failed"))); },
    };
    let (t, p) = data.consume();
    let functor = functor.get_functor();
    Ok(functor(t, p, arg))
}