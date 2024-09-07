use std::{fmt::{Debug, Formatter}, any::Any};
use std::fmt::Write;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use either::Either;

use crate::context::{Context, ContextData, UnprotectedContext};
use crate::policy::{AnyPolicy, NoPolicy, Policy, RefPolicy, OptionPolicy, Reason};
use crate::pcr::PrivacyCriticalRegion;
use crate::pure::PrivacyPureRegion;

use pin_project_lite::pin_project;

// Privacy Container type.
pin_project! {
    pub struct DirectBBox<T, P: Policy> {
        #[pin]
        t: T,
        p: P,
    }
}

// DirectBBox is cloneable if what is inside is cloneable.
impl<T: Clone, P: Policy + Clone> Clone for DirectBBox<T, P> {
    fn clone(&self) -> Self {
        DirectBBox {
            t: self.t.clone(),
            p: self.p.clone(),
        }
    }
}

// Basic API: does not assume anything about data or policy.
// This API moves/consumes the data and policy, or operates on them as refs.
impl<T, P: Policy> DirectBBox<T, P> {
    pub fn new(t: T, p: P) -> Self {
        Self { t, p }
    }

    // Consumes the DirectBBoxes extracting data and policy (private usable only in crate).
    pub(crate) fn consume(self) -> (T, P) {
        (self.t, self.p)
    }
    pub(crate) fn data(&self) -> &T {
        &self.t
    }

    // Into a reference.
    pub fn as_ref(&self) -> DirectBBox<&T, RefPolicy<P>> {
        DirectBBox::new(&self.t, RefPolicy::new(&self.p))
    }

    // Into and from but without the traits (to avoid specialization issues).
    pub fn into_DirectBBox<F, P2: Policy>(self) -> DirectBBox<F, P2>
    where
        T: Into<F>,
        P: Into<P2>,
    {
        DirectBBox {
            t: self.t.into(),
            p: self.p.into(),
        }
    }
    pub fn from_DirectBBox<F>(value: DirectBBox<F, P>) -> DirectBBox<T, P>
    where
        T: From<F>,
    {
        DirectBBox {
            t: T::from(value.t),
            p: value.p,
        }
    }
    // retrieve policy
    pub fn policy(&self) -> &P{
        &self.p
    }

    // Unbox with policy checks.
    pub fn unbox<'a, D: ContextData, C, O, F: FnOnce(&'a T, C) -> O>(
        &'a self,
        context: Context<D>,
        functor: PrivacyCriticalRegion<F>,
        arg: C
    ) -> Result<O, ()> {
        let context = UnprotectedContext::from(context);
        if self.p.check(&context, Reason::Custom) {
            let functor = functor.get_functor();
            Ok(functor(&self.t, arg))
        } else {
            Err(())
        }
    }
    pub fn into_unbox<D: ContextData, C, O, F: FnOnce(T, C) -> O>(
        self,
        context: Context<D>,
        functor: PrivacyCriticalRegion<F>,
        arg: C
    ) -> Result<O, ()> {
        let context = UnprotectedContext::from(context);
        if self.p.check(&context, Reason::Custom) {
            let functor = functor.get_functor();
            Ok(functor(self.t, arg))
        } else {
            Err(())
        }
    }

    // Privacy critical regions
    pub fn pcr<'a, C, O, F: FnOnce(&'a T, &'a P, C) -> O>(&'a self, functor: PrivacyCriticalRegion<F>, arg: C) -> O {
        let functor = functor.get_functor();
        functor(&self.t, &self.p, arg)
    }
    pub fn into_pcr<C, O, F: FnOnce(T, P, C) -> O>(self, functor: PrivacyCriticalRegion<F>, arg: C) -> O {
        let functor = functor.get_functor();
        functor(self.t, self.p, arg)
    }

    // Privacy pure regions.
    pub fn ppr<'a, O, F: FnOnce(&'a T) -> O>(&'a self, functor: PrivacyPureRegion<F>) -> DirectBBox<O, RefPolicy<'a, P>> {
        let functor = functor.get_functor();
        DirectBBox::new(functor(&self.t), RefPolicy::new(&self.p))
    }
    pub fn into_ppr<O, F: FnOnce(T) -> O>(self, functor: PrivacyPureRegion<F>) -> DirectBBox<O, P> {
        let functor = functor.get_functor();
        DirectBBox::new(functor(self.t), self.p)
    }
}

// Can clone a ref policy to own it.
impl<'a, T, P: Policy + Clone> DirectBBox<T, RefPolicy<'a, P>> {
    pub fn to_owned_policy(self) -> DirectBBox<T, P> {
        DirectBBox::new(self.t, self.p.policy().clone())
    }
}

// Can clone a ref to own it.
impl<'r, T: ToOwned + ?Sized, P: Policy + Clone> DirectBBox<&'r T, RefPolicy<'r, P>> {
    pub fn to_owned(&self) -> DirectBBox<T::Owned, P> {
        DirectBBox::new(self.t.to_owned(), self.p.policy().clone())
    }
}

// Up casting to std::any::Any and AnyPolicy.
impl<T: 'static, P: Policy + Clone + 'static> DirectBBox<T, P> {
    pub fn into_any(self) -> DirectBBox<Box<dyn Any>, AnyPolicy> {
        DirectBBox::new(Box::new(self.t),
                  AnyPolicy::new(self.p))
    }
}

// Specializing OptionPolicy.
impl<T, P: Policy + Clone + 'static> DirectBBox<T, OptionPolicy<P>> {
    pub fn specialize(self) -> Either<DirectBBox<T, NoPolicy>, DirectBBox<T, P>> {
        let (t, p) = self.consume();
        match p {
            OptionPolicy::NoPolicy => Either::Left(DirectBBox::new(t, NoPolicy {})),
            OptionPolicy::Policy(p) => Either::Right(DirectBBox::new(t, p)),
        }
    }
}

// Up and downcasting policy with AnyPolicy.
impl<T, P: Policy + Clone + 'static> DirectBBox<T, P> {
    pub fn into_any_policy(self) -> DirectBBox<T, AnyPolicy> {
        DirectBBox {
            t: self.t,
            p: AnyPolicy::new(self.p),
        }
    }
}
impl<T> DirectBBox<T, AnyPolicy> {
    pub fn is_policy<P: Policy + 'static>(&self) -> bool {
        self.p.is::<P>()
    }
    pub fn specialize_policy<P: Policy + 'static>(self) -> Result<DirectBBox<T, P>, String> {
        Ok(DirectBBox {
            t: self.t,
            p: self.p.specialize()?,
        })
    }
}

// NoPolicy can be discarded, logged, etc
impl<T> DirectBBox<T, NoPolicy> {
    pub fn discard_box(self) -> T {
        self.t
    }
}
impl<T: Debug> Debug for DirectBBox<T, NoPolicy> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Box(")?;
        self.t.fmt(f)?;
        f.write_char(')')
    }
}
impl<T: PartialEq> PartialEq for DirectBBox<T, NoPolicy> {
    fn eq(&self, other: &Self) -> bool {
        self.t.eq(&other.t)
    }
}

// Same but for RefPolicy<NoPolicy>
impl<'a, T> DirectBBox<&'a T, RefPolicy<'a, NoPolicy>> {
    pub fn discard_box(self) -> &'a T {
        self.t
    }
}
impl<'a, T: Debug> Debug for DirectBBox<&'a T, RefPolicy<'a, NoPolicy>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Box(")?;
        self.t.fmt(f)?;
        f.write_char(')')
    }
}
impl<'a, T: PartialEq> PartialEq for DirectBBox<&'a T, RefPolicy<'a, NoPolicy>> {
    fn eq(&self, other: &Self) -> bool {
        self.t.eq(&other.t)
    }
}

impl<T, E, P: Policy> DirectBBox<Result<T, E>, P> {
    pub fn transpose(self) -> Result<DirectBBox<T, P>, E> {
        let (t, p) = self.consume();
        Ok(DirectBBox::new(t?, p))
    }
}

impl<T, P: Policy> DirectBBox<Option<T>, P> {
    pub fn transpose(self) -> Option<DirectBBox<T, P>> {
        let (t, p) = self.consume();
        Some(DirectBBox::new(t?, p))
    }
}

impl<'a, T: Future, P: Policy + Clone> Future for DirectBBox<T, P> {
    type Output = DirectBBox<T::Output, P>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.t.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => Poll::Ready(DirectBBox::new(t, this.p.clone())),
        }
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::policy::NoPolicy;
    use crate::testing::{TestContextData, TestPolicy};

    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct ExamplePolicy {
        pub attr: String,
    }
    impl Policy for ExamplePolicy {
        fn name(&self) -> String {
            String::from("ExamplePolicy")
        }
        fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
            true
        }
        fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
            Ok(AnyPolicy::new(self.clone()))
        }
        fn join_logic(&self, _other: Self) -> Result<Self, ()> {
            Ok(ExamplePolicy { attr: String::from("") })
        }
    }

    #[test]
    fn test_box() {
        let DirectBBox = DirectBBox::new(10u64, NoPolicy {});
        assert_eq!(DirectBBox.t, 10u64);
        assert_eq!(DirectBBox.discard_box(), 10u64);
    }

    #[test]
    fn test_unbox() {
        let context = Context::new(
            String::from(""),
            TestContextData::new(()),
        );

        let DirectBBox = DirectBBox::new(10u64, NoPolicy {});
        let result = DirectBBox.into_unbox(
            context,
            PrivacyCriticalRegion::new(|val, exp| {
                assert_eq!(val, exp);
            }),
            10u64);
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_transformation() {
        let DirectBBox = DirectBBox::new(
            String::from("hello"),
            TestPolicy::new(ExamplePolicy {
                attr: String::from("Hello this is a test!"),
            }),
        );
        // Turn it into a Box with a dyn policy.
        let DirectBBox = DirectBBox.into_any_policy();

        // Make sure we can specialize.
        assert!(DirectBBox.is_policy::<TestPolicy<ExamplePolicy>>());
        let DirectBBox = DirectBBox.specialize_policy::<TestPolicy<ExamplePolicy>>().unwrap();

        assert_eq!(DirectBBox.policy().policy().attr, String::from("Hello this is a test!"));
        assert_eq!(DirectBBox.discard_box(), String::from("hello"));
    }

    #[test]
    fn test_into_DirectBBox() {
        let DirectBBox: DirectBBox<u32, NoPolicy> = DirectBBox::new(10u32, NoPolicy {});
        let converted: DirectBBox<u64, NoPolicy> = DirectBBox.into_DirectBBox();
        assert_eq!(converted.discard_box(), 10u64);
    }

    #[test]
    fn test_from_DirectBBox() {
        let DirectBBox: DirectBBox<u32, NoPolicy> = DirectBBox::new(10u32, NoPolicy {});
        let converted = DirectBBox::<u64, NoPolicy>::from_DirectBBox(DirectBBox);
        assert_eq!(converted.discard_box(), 10u64);
    }

    #[test]
    fn test_clone() {
        let DirectBBox = DirectBBox::new(
            String::from("some very long string! hello!!!!"),
            TestPolicy::new(ExamplePolicy {
                attr: String::from("My Policy"),
            }),
        );
        let cloned = DirectBBox.clone();
        assert_eq!(DirectBBox, cloned);
        assert_eq!(DirectBBox.policy().policy().attr, cloned.policy().policy().attr);
        assert_eq!(DirectBBox.discard_box(), cloned.discard_box());
    }
}