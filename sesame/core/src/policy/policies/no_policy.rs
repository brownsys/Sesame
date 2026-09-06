use crate::context::UnprotectedContext;
use crate::pcon::PCon;
use crate::policy::{Policy, Reason};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use serde::ser::SerializeStruct;

// NoPolicy can be directly discarded.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NoPolicy {}

impl NoPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Policy for NoPolicy {
    fn name(&self) -> String {
        String::from("NoPolicy")
    }
    fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
        true
    }
}

impl Default for NoPolicy {
    fn default() -> Self {
        NoPolicy {}
    }
}

// NoPolicy can be discarded, logged, etc
impl<T> PCon<T, NoPolicy> {
    pub fn discard_box(self) -> T {
        self.consume().0
    }

    // Attaches a policy to an unlabelled value without observing it.
    // Only offered on NoPolicy, which restricts nothing, so this can only add
    // restriction and cannot be used to weaken or strip an existing policy.
    // PCon::new(pcon.discard_box(), policy) is equivalent but reads the value,
    // which matters when the policy depends on something other than the value
    // it protects (e.g. a form field whose policy comes from a sibling field).
    pub fn tag<P: Policy>(self, policy: P) -> PCon<T, P> {
        PCon::new(self.consume().0, policy)
    }
}
impl<T: Debug> Debug for PCon<T, NoPolicy> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PCon")
            .field("data", self.data())
            .field("policy", &"NoPolicy")
            .finish()
    }
}
impl<T: PartialEq> PartialEq for PCon<T, NoPolicy> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}


impl<T: Serialize> Serialize for PCon<T, NoPolicy> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bbox_ser = serializer.serialize_struct("PCon", 2)?;
        bbox_ser.serialize_field("fb", self.data())?;
        bbox_ser.serialize_field("p", self.policy())?;
        bbox_ser.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::extensions::{ExtensionContext, SesameExtension};
    use crate::policy::SimplePolicy;
    use crate::testing::TestContextData;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Gated {
        allow: bool,
    }
    impl SimplePolicy for Gated {
        fn simple_name(&self) -> String {
            String::from("Gated")
        }
        fn simple_check(&self, _ctx: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            self.allow
        }
        fn simple_join_direct(&mut self, other: &mut Self) {
            self.allow = self.allow && other.allow;
        }
    }

    struct Identity;
    impl<P: Policy> SesameExtension<String, P, String> for Identity {
        fn apply(&mut self, data: String, _policy: P) -> String {
            data
        }
    }

    // The SesameType derive emits ::sesame:: paths, which do not resolve
    // inside the sesame crate itself.
    fn ctx() -> Context<TestContextData<()>> {
        Context::new(String::from("test"), TestContextData::new(()))
    }

    #[test]
    fn tag_attaches_the_policy() {
        let tagged = PCon::new(String::from("secret"), NoPolicy {}).tag(Gated { allow: true });
        assert!(tagged.policy().allow);
        assert_eq!(tagged.policy().simple_name(), "Gated");
    }

    #[test]
    fn tagged_value_is_checked() {
        let denied = PCon::new(String::from("secret"), NoPolicy {}).tag(Gated { allow: false });
        assert!(denied
            .checked_extension(&mut Identity, &ExtensionContext::new(ctx()), Reason::Response)
            .is_err());

        let allowed = PCon::new(String::from("secret"), NoPolicy {}).tag(Gated { allow: true });
        assert_eq!(
            allowed
                .checked_extension(&mut Identity, &ExtensionContext::new(ctx()), Reason::Response)
                .unwrap(),
            "secret"
        );
    }

    #[test]
    fn tag_preserves_the_value() {
        let tagged = PCon::new(String::from("secret"), NoPolicy {}).tag(Gated { allow: true });
        assert_eq!(
            tagged
                .checked_extension(&mut Identity, &ExtensionContext::new(ctx()), Reason::Response)
                .unwrap(),
            "secret"
        );
    }
}
