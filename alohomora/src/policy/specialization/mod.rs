mod traits;
mod r#enum;

pub use traits::*;
pub use r#enum::*;

// TODO(babman): move these tests somewhere reasonable.
// TODO(babman): add missing tests.
use crate::policy::{AnyPolicyTrait, AnyPolicyable, Policy, PolicyAnd, PolicyDyn, PolicyDynInto, PolicyDynRelation, SimplePolicy};

#[cfg(test)]
mod tests {
    use crate::context::UnprotectedContext;
    use crate::policy::{AnyPolicyBB, AnyPolicyClone, AnyPolicyDyn, Policy, PolicyAnd, Reason, Specializable};
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
        let e = policy.to_specialization_enum().normalize();
        todo!()
        //let p = e.specialize::<Stacked>().map_err(|_| ()).unwrap();
        //println!("{}", p.name());
    }
}