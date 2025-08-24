use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use either::Either;
use mysql::chrono;

use crate::context::{Context, ContextData, UnprotectedContext};
use crate::pcr::PrivacyCriticalRegion;
use crate::policy::{
    AnyPolicyBB, AnyPolicyCC, AnyPolicyClone, AnyPolicyDyn, AnyPolicyable,
    NoPolicy, OptionPolicy, Policy, PolicyDyn, PolicyDynRelation, Reason,
    RefPolicy, Specializable, SpecializationEnum, Specialize,
};
use crate::pure::PrivacyPureRegion;

use crate::bbox::obfuscated_pointer::ObPtr;
use crate::fold::fold;
use crate::SesameType;

use pin_project_lite::pin_project;

// Privacy Container type.
pin_project! {
    pub struct BBox<T, P: Policy> {
        #[pin]
        fb: ObPtr<T>,
        p: P,
    }
}

// BBox is cloneable if what is inside is cloneable.
impl<T: Clone, P: Policy + Clone> Clone for BBox<T, P> {
    fn clone(&self) -> Self {
        BBox {
            fb: self.fb.clone(),
            p: self.p.clone(),
        }
    }
}

// Basic API: does not assume anything about data or policy.
// This API moves/consumes the data and policy, or operates on them as refs.
impl<T, P: Policy> BBox<T, P> {
    pub fn new(t: T, p: P) -> Self {
        Self {
            fb: ObPtr::new(t),
            p,
        }
    }

    // Consumes the bboxes extracting data and policy (private usable only in crate).
    pub(crate) fn consume(self) -> (T, P) {
        (self.fb.mov(), self.p)
    }
    pub(crate) fn data(&self) -> &T {
        self.fb.get()
    }

    // Into a reference.
    pub fn as_ref(&self) -> BBox<&T, RefPolicy<P>> {
        BBox::new(self.fb.get(), RefPolicy::new(&self.p))
    }

    // Into and from but without the traits (to avoid specialization issues).
    pub fn into_bbox<F, P2: Policy>(self) -> BBox<F, P2>
    where
        T: Into<F>,
        P: Into<P2>,
    {
        BBox {
            fb: ObPtr::new((self.fb.mov()).into()),
            p: self.p.into(),
        }
    }
    pub fn from_bbox<F>(value: BBox<F, P>) -> BBox<T, P>
    where
        T: From<F>,
    {
        BBox {
            fb: ObPtr::new(T::from(value.fb.mov())),
            p: value.p,
        }
    }
    // retrieve policy
    pub fn policy(&self) -> &P {
        &self.p
    }

    // Unbox with policy checks.
    pub fn unbox<'a, D: ContextData, C: SesameType, O, F: FnOnce(&'a T, C::Out) -> O>(
        &'a self,
        context: Context<D>,
        functor: PrivacyCriticalRegion<F>,
        arg: C,
    ) -> Result<O, ()>
    where
        C::Out: Any,
    {
        let arg_out = fold(arg).unwrap().consume().0;
        let context = UnprotectedContext::from(context);
        if self.p.check(&context, Reason::Custom(&arg_out)) {
            let functor = functor.get_functor();
            Ok(functor(self.fb.get(), arg_out))
        } else {
            Err(())
        }
    }
    pub fn into_unbox<D: ContextData, C: SesameType, O, F: FnOnce(T, C::Out) -> O>(
        self,
        context: Context<D>,
        functor: PrivacyCriticalRegion<F>,
        arg: C,
    ) -> Result<O, ()>
    where
        C::Out: Any,
    {
        let arg_out = fold(arg).unwrap().consume().0;
        let context = UnprotectedContext::from(context);
        if self.p.check(&context, Reason::Custom(&arg_out)) {
            let functor = functor.get_functor();
            Ok(functor(self.fb.mov(), arg_out))
        } else {
            Err(())
        }
    }

    // Privacy critical regions
    pub fn pcr<'a, C, O, F: FnOnce(&'a T, &'a P, C) -> O>(
        &'a self,
        functor: PrivacyCriticalRegion<F>,
        arg: C,
    ) -> O {
        let functor = functor.get_functor();
        functor(self.fb.get(), &self.p, arg)
    }
    pub fn into_pcr<C, O, F: FnOnce(T, P, C) -> O>(
        self,
        functor: PrivacyCriticalRegion<F>,
        arg: C,
    ) -> O {
        let functor = functor.get_functor();
        functor(self.fb.mov(), self.p, arg)
    }

    // Privacy pure regions.
    pub fn ppr<'a, O, F: FnOnce(&'a T) -> O>(
        &'a self,
        functor: PrivacyPureRegion<F>,
    ) -> BBox<O, RefPolicy<'a, P>> {
        let functor = functor.get_functor();
        BBox::new(functor(self.fb.get()), RefPolicy::new(&self.p))
    }
    pub fn into_ppr<O, F: FnOnce(T) -> O>(self, functor: PrivacyPureRegion<F>) -> BBox<O, P> {
        let functor = functor.get_functor();
        BBox::new(functor(self.fb.mov()), self.p)
    }
}

// Can clone a ref policy to own it.
impl<'a, T, P: Policy + Clone> BBox<T, RefPolicy<'a, P>> {
    pub fn to_owned_policy(self) -> BBox<T, P> {
        BBox {
            fb: self.fb,
            p: self.p.policy().clone(),
        }
    }
}

// Can clone a ref to own it.
impl<'r, T: ToOwned + ?Sized, P: Policy + Clone> BBox<&'r T, RefPolicy<'r, P>> {
    pub fn to_owned(&self) -> BBox<T::Owned, P> {
        BBox::new(self.fb.clone().mov().to_owned(), self.p.policy().clone())
    }
}

// Up casting to std::any::Any and AnyPolicy.
impl<T: Any, P: AnyPolicyClone> BBox<T, P> {
    pub fn into_any_cloneable(self) -> BBox<Box<dyn Any>, AnyPolicyCC> {
        BBox::new(Box::new(self.fb.mov()), AnyPolicyCC::new(self.p))
    }
}
impl<T: Any, P: AnyPolicyable> BBox<T, P> {
    pub fn into_any_no_clone(self) -> BBox<Box<dyn Any>, AnyPolicyBB> {
        BBox::new(Box::new(self.fb.mov()), AnyPolicyBB::new(self.p))
    }
}

// Specializing OptionPolicy.
impl<T, P: AnyPolicyable> BBox<T, OptionPolicy<P>> {
    pub fn specialize_option_policy(self) -> Either<BBox<T, NoPolicy>, BBox<T, P>> {
        let (t, p) = self.consume();
        match p {
            OptionPolicy::NoPolicy => Either::Left(BBox::new(t, NoPolicy {})),
            OptionPolicy::Policy(p) => Either::Right(BBox::new(t, p)),
        }
    }
}

// Upcasting to AnyPolicy.
impl<T, P: AnyPolicyClone> BBox<T, P> {
    pub fn into_any_policy(self) -> BBox<T, AnyPolicyCC> {
        BBox {
            fb: self.fb,
            p: AnyPolicyCC::new(self.p),
        }
    }
}
impl<T, P: AnyPolicyable> BBox<T, P> {
    pub fn into_any_policy_no_clone(self) -> BBox<T, AnyPolicyBB> {
        BBox {
            fb: self.fb,
            p: AnyPolicyBB::new(self.p),
        }
    }
}

// Downcasting to AnyPolicy.
impl<T, DynP: PolicyDyn + ?Sized> BBox<T, AnyPolicyDyn<DynP>> {
    pub fn is_policy<P: AnyPolicyable>(&self) -> bool
    where
        DynP: PolicyDynRelation<P>,
    {
        self.p.is::<P>()
    }

    pub fn specialize_top_policy<P: AnyPolicyable>(self) -> Result<BBox<T, P>, String>
    where
        DynP: PolicyDynRelation<P>,
    {
        Ok(BBox {
            fb: self.fb,
            p: self.p.specialize_top()?,
        })
    }

    pub fn specialize_policy_ref<P: AnyPolicyable>(
        &self,
    ) -> Result<BBox<&T, RefPolicy<'_, P>>, String>
    where
        DynP: PolicyDynRelation<P>,
    {
        Ok(BBox::new(
            self.fb.get(),
            RefPolicy::new(self.p.specialize_top_ref()?),
        ))
    }
}
impl<'a, T, DynP: PolicyDyn + ?Sized> BBox<T, RefPolicy<'a, AnyPolicyDyn<DynP>>> {
    pub fn is_policy_ref<P: AnyPolicyable>(&self) -> bool
    where
        DynP: PolicyDynRelation<P>,
    {
        self.p.policy().is::<P>()
    }

    pub fn specialize_policy_ref<P: AnyPolicyable>(
        &self,
    ) -> Result<BBox<&T, RefPolicy<'_, P>>, String>
    where
        DynP: PolicyDynRelation<P>,
    {
        Ok(BBox::new(
            self.fb.get(),
            RefPolicy::new(self.p.policy().specialize_top_ref()?),
        ))
    }
}

// Normalized specialize over entire policy via reflection.
impl<T, P: Specializable> BBox<T, P> {
    pub fn specialize_policy<P2: Specialize>(
        self,
    ) -> Result<BBox<T, P2>, BBox<T, SpecializationEnum>> {
        let (fb, p) = (self.fb, self.p);
        match p.specialize::<P2>() {
            Ok(p) => Ok(BBox { fb, p }),
            Err(p) => Err(BBox { fb, p }),
        }
    }
}

// Future.
impl<'a, T: Future + Unpin, P: Policy + Clone> Future for BBox<T, P> {
    type Output = BBox<T::Output, P>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.fb.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => Poll::Ready(BBox::new(t, this.p.clone())),
        }
    }
}

// Default.
impl<T: Default, P: Policy + Default> Default for BBox<T, P> {
    fn default() -> Self {
        BBox::new(T::default(), P::default())
    }
}

// For datetime.
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, ParseResult};

impl<P: Policy> BBox<String, P> {
    pub fn into_date_time(self, fmt: &str) -> ParseResult<BBox<NaiveDateTime, P>> {
        let (t, p) = self.consume();
        Ok(BBox::new(NaiveDateTime::parse_from_str(&t, fmt)?, p))
    }
    pub fn into_date(self, fmt: &str) -> ParseResult<BBox<NaiveDate, P>> {
        let (t, p) = self.consume();
        Ok(BBox::new(NaiveDate::parse_from_str(&t, fmt)?, p))
    }
    pub fn into_time(self, fmt: &str) -> ParseResult<BBox<NaiveTime, P>> {
        let (t, p) = self.consume();
        Ok(BBox::new(NaiveTime::parse_from_str(&t, fmt)?, p))
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::policy::{NoPolicy, SimplePolicy};
    use crate::testing::{TestContextData, TestPolicy};

    use super::*;
    use crate::pcr::Signature;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ExamplePolicy {
        pub attr: String,
    }
    impl SimplePolicy for ExamplePolicy {
        fn simple_name(&self) -> String {
            String::from("ExamplePolicy")
        }
        fn simple_check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
            true
        }
        fn simple_join_direct(&mut self, _other: &mut Self) {
            unreachable!()
        }
    }

    #[test]
    fn test_box() {
        let bbox = BBox::new(10u64, NoPolicy {});
        assert_eq!(bbox.fb.get(), &10u64);
        assert_eq!(bbox.discard_box(), 10u64);
    }

    #[test]
    fn test_unbox() {
        let context = Context::new(String::from(""), TestContextData::new(()));

        let bbox = BBox::new(10u64, NoPolicy {});
        let result = bbox.into_unbox(
            context,
            PrivacyCriticalRegion::new(
                |val, exp| {
                    assert_eq!(val, exp);
                },
                Signature {
                    username: "",
                    signature: "",
                },
            ),
            10u64,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_transformation() {
        let bbox = BBox::new(
            String::from("hello"),
            TestPolicy::new(ExamplePolicy {
                attr: String::from("Hello this is a test!"),
            }),
        );
        // Turn it into a Box with a dyn policy.
        let bbox = bbox.into_any_policy();

        // Make sure we can specialize.
        assert!(bbox.is_policy::<TestPolicy<ExamplePolicy>>());
        let bbox = bbox
            .specialize_top_policy::<TestPolicy<ExamplePolicy>>()
            .unwrap();

        assert_eq!(
            bbox.policy().policy().attr,
            String::from("Hello this is a test!")
        );
        assert_eq!(bbox.discard_box(), String::from("hello"));
    }

    #[test]
    fn test_into_bbox() {
        let bbox: BBox<u32, NoPolicy> = BBox::new(10u32, NoPolicy {});
        let converted: BBox<u64, NoPolicy> = bbox.into_bbox();
        assert_eq!(converted.discard_box(), 10u64);
    }

    #[test]
    fn test_from_bbox() {
        let bbox: BBox<u32, NoPolicy> = BBox::new(10u32, NoPolicy {});
        let converted = BBox::<u64, NoPolicy>::from_bbox(bbox);
        assert_eq!(converted.discard_box(), 10u64);
    }

    #[test]
    fn test_clone() {
        let bbox = BBox::new(
            String::from("some very long string! hello!!!!"),
            TestPolicy::new(ExamplePolicy {
                attr: String::from("My Policy"),
            }),
        );
        let cloned = bbox.clone();
        assert_eq!(bbox, cloned);
        assert_eq!(bbox.policy().policy().attr, cloned.policy().policy().attr);
        assert_eq!(bbox.discard_box(), cloned.discard_box());
    }
}
