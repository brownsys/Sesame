use crate::context::Context;
use std::{fmt::{Debug, Display, Formatter}, any::Any};

use crate::policy::{AnyPolicy, NoPolicy, Policy};

pub struct BBox<T, P: Policy> {
    pub(crate) t: T,
    pub(crate) p: P,
}

// Basic API: does not assume anything about data or policy.
// This API moves/consumes the data and policy, or operates on them as refs.
impl<T, P: Policy> BBox<T, P> {
    pub fn new(t: T, p: P) -> Self {
        Self { t, p }
    }

    // Into and from but without the traits (to avoid specialization issues).
    pub fn into_bbox<F>(self) -> BBox<F, P>
    where
        T: Into<F>,
    {
        BBox {
            t: self.t.into(),
            p: self.p,
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
    //retrieve policy
    pub fn policy(&self) -> &P{
        &self.p
    }

    // Unbox with policy checks.
    pub fn temporary_unbox(&self) -> &T {
        &self.t
    }
    pub fn into_temporary_unbox(self) -> T {
        self.t
    }
    pub fn unbox<U, D>(&self, _context: &Context<U, D>) -> &T {
        &self.t
    }
    pub fn into_unbox<U, D>(self, _context: &Context<U, D>) -> T {
        self.t
    }

    // Sandbox functions
    pub fn into_sandbox_execute<R, F: FnOnce(T) -> R>(self, lambda: F) -> BBox<R, P> {
        // Do we check policies?
        // Do we check that function is pure?
        // Do we execute in an actual sandbox?
        BBox {
            t: lambda(self.t),
            p: self.p,
        }
    }
}

// This API assume the policy can be cloned.
impl<T, P: Policy + Clone> BBox<T, P> {
    pub fn sandbox_execute<'a, R, F: FnOnce(&'a T) -> R>(&'a self, lambda: F) -> BBox<R, P> {
        // Do we check policies?
        // Do we check that function is pure?
        // Do we execute in an actual sandbox?
        BBox {
            t: lambda(&self.t),
            p: self.p.clone(),
        }
    }
}

impl<T: 'static, P: Policy + Clone + 'static> BBox<T, P> { 
    pub fn to_any_type_and_policy(self) -> BBox<Box<dyn Any>, AnyPolicy> {
        BBox::new(Box::new(self.t),
                  AnyPolicy::new(self.p))
    }
}

// Up and downcasting policy with AnyPolicy.
impl<T, P: Policy + Clone + 'static> BBox<T, P> {
    pub fn any_policy(self) -> BBox<T, AnyPolicy> {
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

// NoPolicy can be discarded.
impl<T> BBox<T, NoPolicy> {
    pub fn discard_box(self) -> T {
        self.t
    }
}

// Debuggable but in boxed form.
impl<T, P: Policy> Debug for BBox<T, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("<<Boxed Data>>")
    }
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

// Formatting to string.
impl<T: Display, P: Policy + Clone> BBox<T, P> {
    pub fn format(&self) -> BBox<String, P> {
        BBox {
            t: format!("{}", self.t),
            p: self.p.clone(),
        }
    }
}
impl<T: Display, P: Policy> BBox<T, P> {
    pub fn into_format(self) -> BBox<String, P> {
        BBox {
            t: format!("{}", self.t),
            p: self.p,
        }
    }
}

//TODO(corinn) in order to use as_ref() in predict.rs - double check valid
impl<T, P: Policy> AsRef<BBox<T, P>> for BBox<T, P> {
    fn as_ref(&self) -> &BBox<T, P> {
        self
    }
}

// Can unbox without context during tests.
#[cfg(test)]
impl<T, P: Policy> BBox<T, P> {
    pub fn test_unbox(&self) -> &T {
        &self.t
    }
    pub fn into_test_unbox(self) -> T {
        self.t
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::policy::NoPolicy;
    use std::any::Any;

    use super::*;

    #[derive(Clone)]
    struct TestPolicy {
        pub attr: String,
    }
    impl Policy for TestPolicy {
        fn name(&self) -> String {
            String::from("TestPolicy")
        }
        fn check(&self, _context: &dyn Any) -> bool {
            true
        }
    }

    #[test]
    fn test_box() {
        let bbox = BBox::new(10u64, NoPolicy {});
        assert_eq!(bbox.t, 10u64);
    }

    #[test]
    fn test_unbox() {
        let bbox = BBox::new(10u64, NoPolicy {});
        assert_eq!(bbox.into_test_unbox(), 10u64);
    }

    #[test]
    fn test_policy_transformation() {
        let bbox = BBox::new(
            String::from("hello"),
            TestPolicy {
                attr: String::from("Hello this is a test!"),
            },
        );
        // Turn it into a Box with a dyn policy.
        let bbox = bbox.any_policy();

        // Make sure we can specialize.
        assert!(bbox.is_policy::<TestPolicy>());
        let bbox = bbox.specialize_policy::<TestPolicy>().unwrap();

        assert_eq!(bbox.p.attr, String::from("Hello this is a test!"));
        assert_eq!(bbox.into_test_unbox(), String::from("hello"));
    }

    #[test]
    fn test_into_bbox() {
        let bbox: BBox<u32, NoPolicy> = BBox::new(10u32, NoPolicy {});
        let converted: BBox<u64, NoPolicy> = bbox.into_bbox::<u64>();
        assert_eq!(converted.t, 10u64);
    }

    #[test]
    fn test_from_bbox() {
        let bbox: BBox<u32, NoPolicy> = BBox::new(10u32, NoPolicy {});
        let converted = BBox::<u64, NoPolicy>::from_bbox(bbox);
        assert_eq!(converted.t, 10u64);
    }

    #[test]
    fn test_clone() {
        let bbox = BBox::new(
            String::from("some very long string! hello!!!!"),
            TestPolicy {
                attr: String::from("My Policy"),
            },
        );
        let cloned = bbox.clone();
        assert_eq!(bbox.t, cloned.t);
        assert_eq!(bbox.p.attr, cloned.p.attr);
    }
}
