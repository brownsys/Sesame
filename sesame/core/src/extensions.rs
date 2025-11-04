use crate::context::{Context, ContextData, UnprotectedContext};
use crate::error::{SesameError, SesameResult};
use crate::pcon::PCon;
use crate::policy::{Policy, Reason};

// An extension is essentially a specific closure we allow to consume the internals of a PCon and
// return an arbitrary data type (not necessarily protected).
//
// The reasoning behind only offering these two APIs is that the other option suddenly becomes to
// have 6 methods :
// - Owned data and self
// - Owned data, borrowed immut self
// - Owned data, borrowed mutable self
// - Borrowed data, owned self
// - Borrowed data, borrowed immut self
// - Borrowed data, borrowed mutable self
//
// Here instead, extensions can be defined over both base types and their references.
// Because extensions are to be used sparingly and are expected to be written by the Tahini team
// (or at least reviewed by them), we consider this an okay effort.
pub trait SesameExtension<T, P: Policy, R> {
    fn apply(&mut self, data: T, policy: P) -> R;
}
pub trait SesameRefExtension<'a, T, P: Policy, R> {
    fn apply_ref(&mut self, data: &'a T, policy: &'a P) -> R;
}

#[async_trait::async_trait]
pub trait AsyncSesameExtension<T, P: Policy, R> {
    async fn async_apply(&mut self, data: T, policy: P) -> R
    where
        T: 'async_trait,
        P: 'async_trait,
        R: 'async_trait;
}

// Marks unchecked extensions
pub trait UncheckedSesameExtension {}

// Allows unprotecting the context once to use it repeatedly without revealing it to the application.
pub struct ExtensionContext {
    context: UnprotectedContext,
}
impl ExtensionContext {
    pub fn new<D: ContextData>(context: Context<D>) -> Self {
        ExtensionContext {
            context: UnprotectedContext::from(context),
        }
    }
}

// How extension is used.
impl<T, P: Policy> PCon<T, P> {
    // Invoke extension without policy check.
    pub fn unchecked_extension<R, E: SesameExtension<T, P, R> + UncheckedSesameExtension>(
        self,
        extension: &mut E,
    ) -> R {
        let (t, p) = self.consume();
        extension.apply(t, p)
    }
    pub fn unchecked_extension_ref<
        'a,
        R,
        E: SesameRefExtension<'a, T, P, R> + UncheckedSesameExtension,
    >(
        &'a self,
        extension: &mut E,
    ) -> R {
        let t = self.data();
        let p = self.policy();
        extension.apply_ref(t, p)
    }
    pub async fn unchecked_async_extension<
        R,
        E: AsyncSesameExtension<T, P, R> + UncheckedSesameExtension,
    >(
        self,
        extension: &mut E,
    ) -> R {
        let (t, p) = self.consume();
        extension.async_apply(t, p).await
    }

    // Invoke extension after policy check.
    pub fn checked_extension<'a, R, E: SesameExtension<T, P, R>>(
        self,
        extension: &mut E,
        context: &ExtensionContext,
        reason: Reason<'a>,
    ) -> SesameResult<R> {
        let (t, p) = self.consume();
        if p.check(&context.context, reason) {
            Ok(extension.apply(t, p))
        } else {
            Err(SesameError::PolicyCheckFailed(format!(
                "Policy check failed {}",
                p.name()
            )))
        }
    }
    pub fn checked_extension_ref<'a, 'b, R, E: SesameRefExtension<'b, T, P, R>>(
        &'b self,
        extension: &mut E,
        context: &ExtensionContext,
        reason: Reason<'a>,
    ) -> SesameResult<R> {
        let (t, p) = (self.data(), self.policy());
        if p.check(&context.context, reason) {
            Ok(extension.apply_ref(t, p))
        } else {
            Err(SesameError::PolicyCheckFailed(format!(
                "Policy check failed {}",
                p.name()
            )))
        }
    }
    pub async fn checked_async_extension<'a, R, E: AsyncSesameExtension<T, P, R>>(
        self,
        extension: &mut E,
        context: &ExtensionContext,
        reason: Reason<'a>,
    ) -> SesameResult<R> {
        let (t, p) = self.consume();
        if p.check(&context.context, reason) {
            Ok(extension.async_apply(t, p).await)
        } else {
            Err(SesameError::PolicyCheckFailed(format!(
                "Policy check failed {}",
                p.name()
            )))
        }
    }
}
