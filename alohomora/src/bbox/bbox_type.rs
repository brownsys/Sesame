use std::{fmt::{Debug, Formatter}, any::Any};
use std::fmt::Write;

use either::Either;

use crate::context::{Context, ContextData, UnprotectedContext};
use crate::policy::{AnyPolicy, NoPolicy, Policy, RefPolicy, OptionPolicy};
use crate::pcr::PrivacyCriticalRegion;
use crate::pure::PrivacyPureRegion;

// Privacy Container type.
pub struct BBox<T, P: Policy> {
    t: T,
    p: P,
}

// BBox is cloneable if what is inside is cloneable.
impl<T: Clone, P: Policy + Clone> Clone for BBox<T, P> {
    fn clone(&self) -> Self {
        BBox {
            t: self.t.clone(),
            p: self.p.clone(),
        }
    }
}

// Basic API: does not assume anything about data or policy.
// This API moves/consumes the data and policy, or operates on them as refs.
impl<T, P: Policy> BBox<T, P> {
    pub fn new(t: T, p: P) -> Self {
        Self { t, p }
    }

    // Consumes the bboxes extracting data and policy (private usable only in crate).
    pub(crate) fn consume(self) -> (T, P) {
        (self.t, self.p)
    }
    pub(crate) fn data(&self) -> &T {
        &self.t
    }

    // Into a reference.
    pub fn as_ref(&self) -> BBox<&T, RefPolicy<P>> {
        BBox::new(&self.t, RefPolicy::new(&self.p))
    }

    // Into and from but without the traits (to avoid specialization issues).
    pub fn into_bbox<F, P2: Policy>(self) -> BBox<F, P2>
    where
        T: Into<F>,
        P: Into<P2>,
    {
        BBox {
            t: self.t.into(),
            p: self.p.into(),
        }
    }
    pub fn from_bbox<F>(value: BBox<F, P>) -> BBox<T, P>
    where
        T: From<F>,
    {
        BBox {
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
        if self.p.check(&context) {
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
        if self.p.check(&context) {
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
    pub fn ppr<'a, O, F: FnOnce(&'a T) -> O>(&'a self, functor: PrivacyPureRegion<F>) -> BBox<O, RefPolicy<'a, P>> {
        let functor = functor.get_functor();
        BBox::new(functor(&self.t), RefPolicy::new(&self.p))
    }
    pub fn into_ppr<O, F: FnOnce(T) -> O>(self, functor: PrivacyPureRegion<F>) -> BBox<O, P> {
        let functor = functor.get_functor();
        BBox::new(functor(self.t), self.p)
    }
}

// Can clone a ref policy to own it.
impl<'a, T, P: Policy + Clone> BBox<&'a T, RefPolicy<'a, P>> {
    pub fn to_owned_policy(&self) -> BBox<&'a T, P> {
        BBox::new(self.t, self.p.policy().clone())
    }
}

// Can clone a ref to own it.
impl<'r, T: Clone, P: Policy + Clone> BBox<&'r T, RefPolicy<'r, P>> {
    pub fn to_owned(&self) -> BBox<T, P> {
        BBox::new(self.t.clone(), self.p.policy().clone())
    }
}

// Up casting to std::any::Any and AnyPolicy.
impl<T: 'static, P: Policy + Clone + 'static> BBox<T, P> {
    pub fn into_any(self) -> BBox<Box<dyn Any>, AnyPolicy> {
        BBox::new(Box::new(self.t),
                  AnyPolicy::new(self.p))
    }
}

// Specializing OptionPolicy.
impl<T, P: Policy + Clone + 'static> BBox<T, OptionPolicy<P>> {
    pub fn specialize(self) -> Either<BBox<T, NoPolicy>, BBox<T, P>> {
        let (t, p) = self.consume();
        match p {
            OptionPolicy::NoPolicy => Either::Left(BBox::new(t, NoPolicy {})),
            OptionPolicy::Policy(p) => Either::Right(BBox::new(t, p)),
        }
    }
}

// Up and downcasting policy with AnyPolicy.
impl<T, P: Policy + Clone + 'static> BBox<T, P> {
    pub fn into_any_policy(self) -> BBox<T, AnyPolicy> {
        BBox {
            t: self.t,
            p: AnyPolicy::new(self.p),
        }
    }
}
impl<T> BBox<T, AnyPolicy> {
    pub fn is_policy<P: Policy + 'static>(&self) -> bool {
        self.p.is::<P>()
    }
    pub fn specialize_policy<P: Policy + 'static>(self) -> Result<BBox<T, P>, String> {
        Ok(BBox {
            t: self.t,
            p: self.p.specialize()?,
        })
    }
}

// NoPolicy can be discarded, logged, etc
impl<T> BBox<T, NoPolicy> {
    pub fn discard_box(self) -> T {
        self.t
    }
}
impl<T: Debug> Debug for BBox<T, NoPolicy> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Box(")?;
        self.t.fmt(f)?;
        f.write_char(')')
    }
}
impl<T: PartialEq> PartialEq for BBox<T, NoPolicy> {
    fn eq(&self, other: &Self) -> bool {
        self.t.eq(&other.t)
    }
}

// Same but for RefPolicy<NoPolicy>
impl<'a, T> BBox<&'a T, RefPolicy<'a, NoPolicy>> {
    pub fn discard_box(self) -> &'a T {
        self.t
    }
}
impl<'a, T: Debug> Debug for BBox<&'a T, RefPolicy<'a, NoPolicy>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Box(")?;
        self.t.fmt(f)?;
        f.write_char(')')
    }
}
impl<'a, T: PartialEq> PartialEq for BBox<&'a T, RefPolicy<'a, NoPolicy>> {
    fn eq(&self, other: &Self) -> bool {
        self.t.eq(&other.t)
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
        fn check(&self, _context: &UnprotectedContext) -> bool {
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
        let bbox = BBox::new(10u64, NoPolicy {});
        assert_eq!(bbox.t, 10u64);
        assert_eq!(bbox.discard_box(), 10u64);
    }

    #[test]
    fn test_unbox() {
        let context = Context::new(
            String::from(""),
            TestContextData::new(()),
        );

        let bbox = BBox::new(10u64, NoPolicy {});
        let result = bbox.into_unbox(
            context,
            PrivacyCriticalRegion::new(|val, exp| {
                assert_eq!(val, exp);
            }),
            10u64);
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
        let bbox = bbox.specialize_policy::<TestPolicy<ExamplePolicy>>().unwrap();

        assert_eq!(bbox.policy().policy().attr, String::from("Hello this is a test!"));
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