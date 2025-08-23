use std::any::{Any};
use crate::context::UnprotectedContext;
use crate::policy::{Policy, PolicyDyn, Reason, NoPolicy};

// Simplified policy interface that application code can implement.
// Application code should implement this trait unless they have reasons to implement Joinable manually.
// or if their policy is not Any (e.g. has non-static refs).
pub trait SimplePolicy: Send + Sync + Any + NotAPolicyContainer {
    fn simple_name(&self) -> String;
    fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool;
    fn simple_join_direct(&mut self, other: &mut Self);
}

// Every SimplePolicy is automatically a Policy that can be joined with instances of the same
// policy.
impl<P: SimplePolicy> Policy for P {
    fn name(&self) -> String {
        self.simple_name()
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        self.simple_check(context, reason)
    }

    /*
    fn policy_type_enum(&mut self) -> PolicyTypeEnum<'_> {
        PolicyTypeEnum::Leaf(self)
    }

    fn can_join_with(&mut self, p: &PolicyTypeEnum<'_>) -> bool {
        match p {
            PolicyTypeEnum::Leaf(s) => {
                s.upcast_any().is::<P>() || s.upcast_any().is::<NoPolicy>()
            },
            _ => false,
        }
    }

    fn join(&mut self, p: PolicyTypeEnum<'_>) -> bool {
        match p {
            PolicyTypeEnum::Leaf(other) => {
                let other = other.upcast_mut();
                if other.is::<P>() {
                    let other = other.downcast_mut().unwrap();
                    self.simple_join_direct(other);
                    true
                } else if other.is::<NoPolicy>() {
                    true
                } else {
                    false
                }
            },
            _ => false,
        }
    }
     */
}


// This trait marks that a policy type is safe to join with any instance of the same type.
pub trait ReflexiveJoin : Policy {
    fn reflexive_join(&mut self, other: &mut Self);
}
impl<P: SimplePolicy> ReflexiveJoin for P {
    fn reflexive_join(&mut self, other: &mut Self) {
        self.simple_join_direct(other)
    }
}

// Use this macro to make your policy unjoinable and force stacking.
// TODO(babman): Use Leaf when possible, NoReflection otherwise.
#[macro_export]
macro_rules! Unjoinable {
    () => {
        fn policy_type_enum(&mut self) -> $crate::policy::PolicyTypeEnum<'_> {
            $crate::policy::PolicyTypeEnum::Leaf(self)
        }
        fn can_join_with(&mut self, _p: &$crate::policy::PolicyTypeEnum<'_>) -> bool {
            false
        }
        fn join(&mut self, _p: $crate::policy::PolicyTypeEnum<'_>) -> bool {
            false
        }
    };
    (!Any) => {
        /*
        fn policy_type_enum(&mut self) -> $crate::policy::PolicyTypeEnum<'_> {
            $crate::policy::PolicyTypeEnum::NoReflection
        }
        fn can_join_with(&mut self, _p: &$crate::policy::PolicyTypeEnum<'_>) -> bool {
            false
        }
        fn join(&mut self, _p: $crate::policy::PolicyTypeEnum<'_>) -> bool {
            false
        }
         */
    }
}

pub use Unjoinable;
use crate::policy::NotAPolicyContainer;