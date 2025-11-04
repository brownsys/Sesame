use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use either::Either;

use crate::context::{Context, ContextData, UnprotectedContext};
use crate::critical::{CriticalRegion, UncheckedCriticalRegion};
use crate::policy::{
    AnyPolicy, AnyPolicyClone, AnyPolicyCloneDyn, AnyPolicyable, NoPolicy, OptionPolicy, Policy,
    PolicyDyn, PolicyDynRelation, Reason, RefPolicy, Specializable, SpecializationEnum, Specialize,
};
use crate::verified::VerifiedRegion;

use crate::fold::fold;
use crate::pcon::obfuscated_pointer::ObPtr;
use crate::SesameType;

use pin_project_lite::pin_project;

// Privacy Container type.
pin_project! {
    pub struct PCon<T, P: Policy> {
        #[pin]
        fb: ObPtr<T>,
        p: P,
    }
}

// PCon is cloneable if what is inside is cloneable.
impl<T: Clone, P: Policy + Clone> Clone for PCon<T, P> {
    fn clone(&self) -> Self {
        PCon {
            fb: self.fb.clone(),
            p: self.p.clone(),
        }
    }
}

// Basic API: does not assume anything about data or policy.
// This API moves/consumes the data and policy, or operates on them as refs.
impl<T, P: Policy> PCon<T, P> {
    pub fn new(t: T, p: P) -> Self {
        Self {
            fb: ObPtr::new(t),
            p,
        }
    }

    // Consumes the pcons extracting data and policy (private usable only in crate).
    pub(crate) fn consume(self) -> (T, P) {
        (self.fb.mov(), self.p)
    }
    pub(crate) fn data(&self) -> &T {
        self.fb.get()
    }

    // Into a reference.
    pub fn as_ref(&self) -> PCon<&T, RefPolicy<P>> {
        PCon::new(self.fb.get(), RefPolicy::new(&self.p))
    }
    pub fn as_ref_pcon<F: ?Sized>(&self) -> PCon<&F, RefPolicy<P>>
    where
        T: AsRef<F>,
    {
        PCon::new(self.fb.get().as_ref(), RefPolicy::new(&self.p))
    }

    // Into and from but without the traits (to avoid specialization issues).
    pub fn into_pcon<F, P2: Policy>(self) -> PCon<F, P2>
    where
        T: Into<F>,
        P: Into<P2>,
    {
        PCon {
            fb: ObPtr::new((self.fb.mov()).into()),
            p: self.p.into(),
        }
    }
    pub fn from_pcon<F>(value: PCon<F, P>) -> PCon<T, P>
    where
        T: From<F>,
    {
        PCon {
            fb: ObPtr::new(T::from(value.fb.mov())),
            p: value.p,
        }
    }
    // retrieve policy
    pub fn policy(&self) -> &P {
        &self.p
    }

    // Privacy pure regions.
    pub fn verified<O, F: FnOnce(&'_ T) -> O>(
        &self,
        functor: VerifiedRegion<F>,
    ) -> PCon<O, RefPolicy<'_, P>> {
        let functor = functor.get_functor();
        PCon::new(functor(self.fb.get()), RefPolicy::new(&self.p))
    }
    pub fn into_verified<O, F: FnOnce(T) -> O>(self, functor: VerifiedRegion<F>) -> PCon<O, P> {
        let functor = functor.get_functor();
        PCon::new(functor(self.fb.mov()), self.p)
    }

    // Critical region with a policy check.
    pub fn critical<D: ContextData, C: SesameType, O, F: FnOnce(&'_ T, C::Out) -> O>(
        &self,
        context: Context<D>,
        functor: CriticalRegion<F>,
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
    pub fn into_critical<D: ContextData, C: SesameType, O, F: FnOnce(T, C::Out) -> O>(
        self,
        context: Context<D>,
        functor: CriticalRegion<F>,
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

    // Critical region without a policy check.
    // THIS IS A LAST RESORT, PREFER TO USE critical() and into_critical() INSTEAD.
    pub fn critical_unchecked<C, O, F: FnOnce(&'_ T, &'_ P, C) -> O>(
        &self,
        functor: UncheckedCriticalRegion<F>,
        arg: C,
    ) -> O {
        let functor = functor.get_functor();
        functor(self.fb.get(), &self.p, arg)
    }
    pub fn into_critical_unchecked<C, O, F: FnOnce(T, P, C) -> O>(
        self,
        functor: UncheckedCriticalRegion<F>,
        arg: C,
    ) -> O {
        let functor = functor.get_functor();
        functor(self.fb.mov(), self.p, arg)
    }
}

// Can clone a ref policy to own it.
impl<'a, T, P: Policy + Clone> PCon<T, RefPolicy<'a, P>> {
    pub fn to_owned_policy(self) -> PCon<T, P> {
        PCon {
            fb: self.fb,
            p: self.p.policy().clone(),
        }
    }
}

// Can clone a ref to own it.
impl<'r, T: ToOwned + ?Sized, P: Policy + Clone> PCon<&'r T, RefPolicy<'r, P>> {
    pub fn to_owned(&self) -> PCon<T::Owned, P> {
        PCon::new(self.fb.clone().mov().to_owned(), self.p.policy().clone())
    }
}

// Up casting to std::any::Any and AnyPolicy.
impl<T: Any, P: AnyPolicyCloneDyn> PCon<T, P> {
    pub fn into_any_cloneable(self) -> PCon<Box<dyn Any>, AnyPolicyClone> {
        PCon::new(Box::new(self.fb.mov()), AnyPolicyClone::new(self.p))
    }
}
impl<T: Any, P: AnyPolicyable> PCon<T, P> {
    pub fn into_any_no_clone(self) -> PCon<Box<dyn Any>, AnyPolicy> {
        PCon::new(Box::new(self.fb.mov()), AnyPolicy::new(self.p))
    }
}

// Specializing OptionPolicy.
impl<T, P: AnyPolicyable> PCon<T, OptionPolicy<P>> {
    pub fn specialize_option_policy(self) -> Either<PCon<T, NoPolicy>, PCon<T, P>> {
        let (t, p) = self.consume();
        match p {
            OptionPolicy::NoPolicy => Either::Left(PCon::new(t, NoPolicy {})),
            OptionPolicy::Policy(p) => Either::Right(PCon::new(t, p)),
        }
    }
}

// Upcasting to AnyPolicy.
impl<T, P: AnyPolicyCloneDyn> PCon<T, P> {
    pub fn into_any_policy(self) -> PCon<T, AnyPolicyClone> {
        PCon {
            fb: self.fb,
            p: AnyPolicyClone::new(self.p),
        }
    }
}
impl<T, P: AnyPolicyable> PCon<T, P> {
    pub fn into_any_policy_no_clone(self) -> PCon<T, AnyPolicy> {
        PCon {
            fb: self.fb,
            p: AnyPolicy::new(self.p),
        }
    }
}

// Downcasting to AnyPolicy.
impl<T, DynP: PolicyDyn + ?Sized> PCon<T, AnyPolicy<DynP>> {
    pub fn is_policy<P: AnyPolicyable>(&self) -> bool
    where
        DynP: PolicyDynRelation<P>,
    {
        self.p.is::<P>()
    }

    pub fn specialize_top_policy<P: AnyPolicyable>(self) -> Result<PCon<T, P>, String>
    where
        DynP: PolicyDynRelation<P>,
    {
        Ok(PCon {
            fb: self.fb,
            p: self.p.specialize_top()?,
        })
    }

    pub fn specialize_policy_ref<P: AnyPolicyable>(
        &self,
    ) -> Result<PCon<&T, RefPolicy<'_, P>>, String>
    where
        DynP: PolicyDynRelation<P>,
    {
        Ok(PCon::new(
            self.fb.get(),
            RefPolicy::new(self.p.specialize_top_ref()?),
        ))
    }
}
impl<'a, T, DynP: PolicyDyn + ?Sized> PCon<T, RefPolicy<'a, AnyPolicy<DynP>>> {
    pub fn is_policy_ref<P: AnyPolicyable>(&self) -> bool
    where
        DynP: PolicyDynRelation<P>,
    {
        self.p.policy().is::<P>()
    }

    pub fn specialize_policy_ref<P: AnyPolicyable>(
        &self,
    ) -> Result<PCon<&T, RefPolicy<'_, P>>, String>
    where
        DynP: PolicyDynRelation<P>,
    {
        Ok(PCon::new(
            self.fb.get(),
            RefPolicy::new(self.p.policy().specialize_top_ref()?),
        ))
    }
}

// Normalized specialize over entire policy via reflection.
impl<T, P: Specializable> PCon<T, P> {
    pub fn specialize_policy<P2: Specialize>(
        self,
    ) -> Result<PCon<T, P2>, PCon<T, SpecializationEnum>> {
        let (fb, p) = (self.fb, self.p);
        match p.specialize::<P2>() {
            Ok(p) => Ok(PCon { fb, p }),
            Err(p) => Err(PCon { fb, p }),
        }
    }
}

// Downcast data if the data is Any.
impl<P: Policy> PCon<Box<dyn Any>, P> {
    pub fn downcast_data<T: Any>(self) -> Result<PCon<T, P>, Self> {
        let (t, p) = self.consume();
        match t.downcast() {
            Ok(t) => Ok(PCon::new(*t, p)),
            Err(t) => Err(PCon::new(t, p)),
        }
    }
}

#[cfg(not(feature = "orm"))]
impl<T> std::fmt::Debug for PCon<T, SpecializationEnum> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.policy().fmt(f)
    }
}

// Future.
impl<'a, T: Future + Unpin, P: Policy + Clone> Future for PCon<T, P> {
    type Output = PCon<T::Output, P>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.fb.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => Poll::Ready(PCon::new(t, this.p.clone())),
        }
    }
}

// Default.
impl<T: Default, P: Policy + Default> Default for PCon<T, P> {
    fn default() -> Self {
        PCon::new(T::default(), P::default())
    }
}

// For datetime.
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, ParseResult};

impl<P: Policy> PCon<String, P> {
    pub fn into_date_time(self, fmt: &str) -> ParseResult<PCon<NaiveDateTime, P>> {
        let (t, p) = self.consume();
        Ok(PCon::new(NaiveDateTime::parse_from_str(&t, fmt)?, p))
    }
    pub fn into_date(self, fmt: &str) -> ParseResult<PCon<NaiveDate, P>> {
        let (t, p) = self.consume();
        Ok(PCon::new(NaiveDate::parse_from_str(&t, fmt)?, p))
    }
    pub fn into_time(self, fmt: &str) -> ParseResult<PCon<NaiveTime, P>> {
        let (t, p) = self.consume();
        Ok(PCon::new(NaiveTime::parse_from_str(&t, fmt)?, p))
    }
}

// A type that contains either T or PCon<T>.
pub type EitherPCon<T, P> = Either<T, PCon<T, P>>;

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::policy::{NoPolicy, SimplePolicy};
    use crate::testing::{TestContextData, TestPolicy};

    use super::*;
    use crate::critical::Signature;

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
        let pcon = PCon::new(10u64, NoPolicy {});
        assert_eq!(pcon.fb.get(), &10u64);
        assert_eq!(pcon.discard_box(), 10u64);
    }

    #[test]
    fn test_unbox() {
        let context = Context::new(String::from(""), TestContextData::new(()));

        let pcon = PCon::new(10u64, NoPolicy {});
        let result = pcon.into_critical(
            context,
            CriticalRegion::new(
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
        let pcon = PCon::new(
            String::from("hello"),
            TestPolicy::new(ExamplePolicy {
                attr: String::from("Hello this is a test!"),
            }),
        );
        // Turn it into a Box with a dyn policy.
        let pcon = pcon.into_any_policy();

        // Make sure we can specialize.
        assert!(pcon.is_policy::<TestPolicy<ExamplePolicy>>());
        let pcon = pcon
            .specialize_top_policy::<TestPolicy<ExamplePolicy>>()
            .unwrap();

        assert_eq!(
            pcon.policy().policy().attr,
            String::from("Hello this is a test!")
        );
        assert_eq!(pcon.discard_box(), String::from("hello"));
    }

    #[test]
    fn test_into_pcon() {
        let pcon: PCon<u32, NoPolicy> = PCon::new(10u32, NoPolicy {});
        let converted: PCon<u64, NoPolicy> = pcon.into_pcon();
        assert_eq!(converted.discard_box(), 10u64);
    }

    #[test]
    fn test_from_pcon() {
        let pcon: PCon<u32, NoPolicy> = PCon::new(10u32, NoPolicy {});
        let converted = PCon::<u64, NoPolicy>::from_pcon(pcon);
        assert_eq!(converted.discard_box(), 10u64);
    }

    #[test]
    fn test_clone() {
        let pcon = PCon::new(
            String::from("some very long string! hello!!!!"),
            TestPolicy::new(ExamplePolicy {
                attr: String::from("My Policy"),
            }),
        );
        let cloned = pcon.clone();
        assert_eq!(pcon, cloned);
        assert_eq!(pcon.policy().policy().attr, cloned.policy().policy().attr);
        assert_eq!(pcon.discard_box(), cloned.discard_box());
    }
}
