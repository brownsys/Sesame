// TODO(babman): Move these tests under conjunction/*.rs and add missing cases.
#[cfg(test)]
mod tests {
    use serde::Serialize;
    use crate::context::UnprotectedContext;
    use crate::policy::{join_dyn, Reason, SimplePolicy, AnyPolicySerialize, AnyPolicyClone, join, AnyPolicyBB, AnyPolicyTrait, Policy};
    use crate::Unjoinable;

    // Simple and easy to join policies.
    #[derive(Serialize, Clone)]
    struct SimplePolicy1 {
        pub k: u32,
    }
    impl SimplePolicy for SimplePolicy1 {
        fn simple_name(&self) -> String {
            String::from("SimplePolicy1")
        }
        fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
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
        fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
            true
        }
        fn simple_join_direct(&mut self, other: &mut Self) {
            self.s = format!("{}:{}", self.s, &other.s);
        }
    }

    // This cannot be joined with other policies.
    struct NoJoinPolicy {}
    impl Policy for NoJoinPolicy {
        fn name(&self) -> String {
            String::from("NoJoinPolicy")
        }
        fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
            true
        }
        Unjoinable!(!Any);
    }

    // Test joining simple policies directly.
    #[test]
    fn test_join_simple1() {
        let simple1 = SimplePolicy1 { k: 1 };
        let simple2 = SimplePolicy1 { k: 3 };
        let simple3 = SimplePolicy1 { k: 8 };

        let joined = join_dyn::<_, _, dyn AnyPolicySerialize>(simple1, simple2);
        let json = serde_json::ser::to_string(&joined);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("{\"policy\":{\"k\":4}}"));

        assert!(joined.is::<SimplePolicy1>());
        let joined: SimplePolicy1 = joined.specialize_top().unwrap();
        assert_eq!(joined.k, 4);

        let joined = join_dyn::<_, _, dyn AnyPolicySerialize>(joined, simple3);
        let json = serde_json::ser::to_string(&joined);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("{\"policy\":{\"k\":12}}"));

        assert!(joined.is::<SimplePolicy1>());
        let joined: SimplePolicy1 = joined.specialize_top().unwrap();
        assert_eq!(joined.k, 12);
    }

    #[test]
    fn test_join_simple2() {
        let simple1 = SimplePolicy2 { s: String::from("hi") };
        let simple2 = SimplePolicy2 { s: String::from("bye") };
        let simple3 = SimplePolicy2 { s: String::from("good") };

        let joined = join_dyn::<_, _, dyn AnyPolicyClone>(simple1, simple2);
        assert!(joined.is::<SimplePolicy2>());
        let joined: SimplePolicy2 = joined.specialize_top().unwrap();
        assert_eq!(joined.s, "hi:bye");

        let joined = join(joined, simple3);
        assert!(joined.is::<SimplePolicy2>());
        let joined: SimplePolicy2 = joined.specialize_top().unwrap();
        assert_eq!(joined.s, "hi:bye:good");
    }

    #[test]
    fn test_join_any_and_any() {
        let simple1 = AnyPolicyBB::new(SimplePolicy1 { k: 5 });
        let simple2 = AnyPolicyBB::new(SimplePolicy1 { k: 7 });
        let joined = join_dyn::<_, _, dyn AnyPolicyTrait>(simple1, simple2);
        assert!(joined.is::<SimplePolicy1>());
        let joined: SimplePolicy1 = joined.specialize_top().unwrap();
        assert_eq!(joined.k, 12);
    }

    #[test]
    fn test_join_simple2_via_any() {
        let simple1 = SimplePolicy2 { s: String::from("hi") };
        let simple2 = SimplePolicy2 { s: String::from("bye") };
        let simple3 = SimplePolicy2 { s: String::from("good") };

        let joined = join_dyn::<_, _, dyn AnyPolicyClone>(simple1, simple2);
        assert!(joined.is::<SimplePolicy2>());
        let joined2: SimplePolicy2 = joined.clone().specialize_top().unwrap();
        assert_eq!(joined2.s, "hi:bye");

        let joined = join(joined, simple3);
        assert!(joined.is::<SimplePolicy2>());
        let joined: SimplePolicy2 = joined.specialize_top().unwrap();
        assert_eq!(joined.s, "hi:bye:good");
    }
}