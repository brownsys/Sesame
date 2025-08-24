use crate::policy::{
    AnyPolicyBB, AnyPolicyDyn, AnyPolicyable, PolicyAnd, PolicyDyn, PolicyDynRelation,
};

// Helper functions.
fn join_helper<P1: AnyPolicyable, P2: AnyPolicyable>(p1: &mut P1, p2: &mut P2) -> bool {
    p1.join_via_reflection(p2.reflect_mut_ref().normalize())
}

// API will be available automatically on qualified types.
// Application developers should use this API.
pub trait JoinAPI: AnyPolicyable + Sized {
    // Developers and rest of Sesame should use this API.
    fn join<P2: AnyPolicyable>(self, other: P2) -> AnyPolicyBB;
    fn join_dyn<P2: AnyPolicyable, PDyn: PolicyDyn + ?Sized>(self, other: P2) -> AnyPolicyDyn<PDyn>
    where
        PDyn: PolicyDynRelation<Self>,
        PDyn: PolicyDynRelation<P2>;
}
impl<P: AnyPolicyable> JoinAPI for P {
    fn join<P2: AnyPolicyable>(self, other: P2) -> AnyPolicyBB {
        self.join_dyn(other)
    }
    fn join_dyn<P2: AnyPolicyable, PDyn: PolicyDyn + ?Sized>(
        mut self,
        mut p2: P2,
    ) -> AnyPolicyDyn<PDyn>
    where
        PDyn: PolicyDynRelation<Self>,
        PDyn: PolicyDynRelation<P2>,
    {
        // Try to join first direction.
        if join_helper(&mut self, &mut p2) {
            return AnyPolicyDyn::new(self);
        }

        // Try to join second direction.
        if join_helper(&mut p2, &mut self) {
            return AnyPolicyDyn::new(p2);
        }

        // Stack.
        PDyn::and_policy(PolicyAnd::new(
            AnyPolicyDyn::new(self),
            AnyPolicyDyn::new(p2),
        ))
    }
}

// Helper used only inside Sesame to deal with join certain type erased policies during fold.
// Specifically: this function is never needed if the PDyn is concrete, because then the PolicyDynRelation bounds
// on the previous API become resolved.
// This is only needed for higher-order generic code where PolicyDynRelation cannot be resolved.
pub fn join_dyn<PDyn: PolicyDyn + ?Sized>(
    mut p1: AnyPolicyDyn<PDyn>,
    mut p2: AnyPolicyDyn<PDyn>,
) -> AnyPolicyDyn<PDyn> {
    // Try to join first direction.
    if join_helper(&mut p1, &mut p2) {
        return p1;
    }

    // Try to join second direction.
    if join_helper(&mut p2, &mut p1) {
        return p2;
    }

    // Stack.
    PDyn::and_policy(PolicyAnd::new(p1, p2))
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::context::UnprotectedContext;
    use crate::policy::{
        AnyPolicyBB, AnyPolicyClone, AnyPolicySerialize, AnyPolicyTrait, Join, JoinAPI, Policy,
        Reason, SimplePolicy,
    };

    // Simple and easy to join policies.
    #[derive(Serialize, Clone)]
    struct SimplePolicy1 {
        pub k: u32,
    }
    impl SimplePolicy for SimplePolicy1 {
        fn simple_name(&self) -> String {
            String::from("SimplePolicy1")
        }
        fn simple_check(&self, _context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            true
        }
        fn simple_join_direct(&mut self, other: &mut Self) {
            self.k = self.k + other.k;
        }
    }

    #[derive(Serialize, Clone)]
    struct SimplePolicy2 {
        pub s: String,
    }
    impl SimplePolicy for SimplePolicy2 {
        fn simple_name(&self) -> String {
            String::from("SimplePolicy2")
        }
        fn simple_check(&self, _context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            true
        }
        fn simple_join_direct(&mut self, other: &mut Self) {
            self.s = format!("{}:{}", self.s, &other.s);
        }
    }

    // This cannot be joined with other policies.
    struct NoJoinPolicy {}
    impl Join for NoJoinPolicy {}
    impl Policy for NoJoinPolicy {
        fn name(&self) -> String {
            String::from("NoJoinPolicy")
        }
        fn check(&self, _context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            true
        }
    }

    // Test joining simple policies directly.
    #[test]
    fn test_join_simple1() {
        let simple1 = SimplePolicy1 { k: 1 };
        let simple2 = SimplePolicy1 { k: 3 };
        let simple3 = SimplePolicy1 { k: 8 };

        let joined = simple1.join_dyn::<_, dyn AnyPolicySerialize>(simple2);
        let json = serde_json::ser::to_string(&joined);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("{\"policy\":{\"k\":4}}"));

        assert!(joined.is::<SimplePolicy1>());
        let joined: SimplePolicy1 = joined.specialize_top().unwrap();
        assert_eq!(joined.k, 4);

        let joined = joined.join_dyn::<_, dyn AnyPolicySerialize>(simple3);
        let json = serde_json::ser::to_string(&joined);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("{\"policy\":{\"k\":12}}"));

        assert!(joined.is::<SimplePolicy1>());
        let joined: SimplePolicy1 = joined.specialize_top().unwrap();
        assert_eq!(joined.k, 12);
    }

    #[test]
    fn test_join_simple2() {
        let simple1 = SimplePolicy2 {
            s: String::from("hi"),
        };
        let simple2 = SimplePolicy2 {
            s: String::from("bye"),
        };
        let simple3 = SimplePolicy2 {
            s: String::from("good"),
        };

        let joined = simple1.join_dyn::<_, dyn AnyPolicyClone>(simple2);
        assert!(joined.is::<SimplePolicy2>());
        let joined: SimplePolicy2 = joined.specialize_top().unwrap();
        assert_eq!(joined.s, "hi:bye");

        let joined = joined.join(simple3);
        assert!(joined.is::<SimplePolicy2>());
        let joined: SimplePolicy2 = joined.specialize_top().unwrap();
        assert_eq!(joined.s, "hi:bye:good");
    }

    #[test]
    fn test_join_any_and_any() {
        let simple1 = AnyPolicyBB::new(SimplePolicy1 { k: 5 });
        let simple2 = AnyPolicyBB::new(SimplePolicy1 { k: 7 });
        let joined = simple1.join_dyn::<_, dyn AnyPolicyTrait>(simple2);
        assert!(joined.is::<SimplePolicy1>());
        let joined: SimplePolicy1 = joined.specialize_top().unwrap();
        assert_eq!(joined.k, 12);
    }

    #[test]
    fn test_join_simple2_via_any() {
        let simple1 = SimplePolicy2 {
            s: String::from("hi"),
        };
        let simple2 = SimplePolicy2 {
            s: String::from("bye"),
        };
        let simple3 = SimplePolicy2 {
            s: String::from("good"),
        };

        let joined = simple1.join_dyn::<_, dyn AnyPolicyClone>(simple2);
        assert!(joined.is::<SimplePolicy2>());
        let joined2: SimplePolicy2 = joined.clone().specialize_top().unwrap();
        assert_eq!(joined2.s, "hi:bye");

        let joined = joined.join(simple3);
        assert!(joined.is::<SimplePolicy2>());
        let joined: SimplePolicy2 = joined.specialize_top().unwrap();
        assert_eq!(joined.s, "hi:bye:good");
    }
}
