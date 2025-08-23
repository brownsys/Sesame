mod traits;
mod r#enum;
mod specialization;

pub use traits::*;
pub use r#enum::*;
pub use specialization::*;

// TODO(babman): move these tests somewhere reasonable.
// TODO(babman): add missing tests.
use crate::policy::{AnyPolicyTrait, AnyPolicyable, Policy, PolicyAnd, PolicyDyn, PolicyDynInto, PolicyDynRelation, SimplePolicy};

#[cfg(test)]
mod tests {
    use crate::context::UnprotectedContext;
    use crate::policy::{AnyPolicyBB, AnyPolicyClone, AnyPolicyDyn, NoPolicy, Policy, PolicyAnd, Reason, RefPolicy, Reflective, ReflectiveOwned, Specializable};
    use crate::Unjoinable;

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Unjoinable {
        pub v: u32,
    }
    impl Policy for Unjoinable {
        fn name(&self) -> String { format!("Unjoinable(v: {})", self.v) }
        fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool { true }
        // This policy is unjoinable.
        Unjoinable!(!Any);
    }

    #[test]
    fn my_special_test() {
        let policy = AnyPolicyDyn::<dyn AnyPolicyClone>::new(
            AnyPolicyDyn::<dyn AnyPolicyClone>::new(
                PolicyAnd::new(
                    AnyPolicyDyn::<dyn AnyPolicyClone>::new(
                        PolicyAnd::new(
                            AnyPolicyDyn::<dyn AnyPolicyClone>::new(Unjoinable { v: 0 }),
                            AnyPolicyDyn::<dyn AnyPolicyClone>::new(Unjoinable { v: 50 }),
                        )
                    ),
                    Unjoinable { v: 20 },
                )
            )
        );

        println!("{}", policy.name());

        type Stacked = PolicyAnd<PolicyAnd<AnyPolicyBB, Unjoinable>, Unjoinable>;
        let e = policy.reflect_owned().normalize();
        let p = e.specialize::<Stacked>().map_err(|_| ()).unwrap();
        println!("{}", p.name());
    }

    #[test]
    fn specialize_policy_ref() {
        let policy = NoPolicy {};
        let policy2 = Unjoinable { v: 50 };
        let policy3 = PolicyAnd::new(policy, policy2);
        let refpolicy: RefPolicy<'static, PolicyAnd<NoPolicy, Unjoinable>> = RefPolicy::new(unsafe {std::mem::transmute(&policy3) });

        let anypolicy = AnyPolicyBB::new(refpolicy);
        type Reffed = RefPolicy<'static, PolicyAnd<NoPolicy, Unjoinable>>;
        let reffed = anypolicy.specialize::<Reffed>().map_err(|_| ()).unwrap();
        assert_eq!(reffed.policy().policy2().v, 50);
    }
}