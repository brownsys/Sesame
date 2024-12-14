use alohomora_policy::k_anonymity_policy;
use alohomora::policy::{Policy, SchemaPolicy, AnyPolicy};
const MIN_K: u64 = 10;

alohomora_policy::k_anonymity_policy!(
    KAnonymityPolicy,
    MIN_K,
    [
        (table: "agg_gender", column: 1),
        (table: "agg_remote", column: 1)
    ]
);

alohomora_policy::k_anonymity_policy!(
    CustomKAnonymityPolicy,
    MIN_K,
    [
        (table: "customers", column: 2)
    ],
    {
        fn from_row(table: &str, row: &Vec<mysql::Value>) -> Self {
            let mut p = Self::new();
            let idx = *p.schema.get(table).expect("Table not in schema");
            p.count = mysql::from_value(row[idx].clone());
            p
        }
    }
);
    

#[cfg(test)]
mod tests {
    use super::*;
    use alohomora::context::UnprotectedContext;
    use alohomora::policy::{AnyPolicy, Policy, Reason};
    use mysql::Value;

    #[test]
    fn test_kanon_policy() {
        let mut policy = KAnonymityPolicy::new();

        let context = UnprotectedContext {
            route: "/test".to_string(),
            data: Box::new(None::<()>),
        };

        // Simulate a row from 'agg_gender' table, with count at column 1:
        let row = vec![mysql::Value::from(42u64), mysql::Value::from(20u64)];
        policy.initialize_from_row("agg_gender", &row).unwrap();

        // Check if the policy meets k-anonymity
        assert!(policy.check(&context, Reason::Response), "count=20 >= 10 should pass");

        // Test with another table 'agg_remote'
        let mut policy2 = KAnonymityPolicy::new();
        let row2 = vec![mysql::Value::from(100u64), mysql::Value::from(15u64)];
        policy2.initialize_from_row("agg_remote", &row2).unwrap();
        assert!(policy2.check(&context, Reason::Response), "count=15 >= 10 should pass");

        // Join two policies and check
        let joined = policy.join(AnyPolicy::new(policy2)).unwrap();
        let joined = joined.specialize::<KAnonymityPolicy>().unwrap();
        // After joining, the count should be the minimum of (20 and 15), which is 15
        assert!(joined.check(&context, Reason::Response));
    }

    #[test]
    fn test_general_kanon_policy() {
        // For testing purposes
        let test_context = UnprotectedContext {
            route: "/test_route".to_string(),
            data: Box::new(None::<()>),
        };

        // Generate General Policy
        alohomora_policy::k_anonymity_policy!(
            GeneralKAnonPolicy,
            10, // MIN_K value
            [
                (table: "agg_gender", column: 1),
                (table: "agg_remote", column: 1),
                (table: "agg_location", column: 1)
            ]
        );

        // Initialize policies from rows representing different tables:
        // For "agg_gender", count is at column index 1:
        let row_gender = vec![Value::from(5u64), Value::from(12u64)];
        let mut policy_gender = GeneralKAnonPolicy::new();
        policy_gender
            .initialize_from_row("agg_gender", &row_gender)
            .expect("Failed to initialize from agg_gender");

        assert!(
            policy_gender.check(&test_context, Reason::Response),
            "count=12 >= 10 should pass"
        );

        // For "agg_remote", put a smaller count that doesn't meet k-anonymity:
        let row_remote = vec![Value::from(100u64), Value::from(8u64)];
        let mut policy_remote = GeneralKAnonPolicy::new();
        policy_remote
            .initialize_from_row("agg_remote", &row_remote)
            .expect("Failed to initialize from agg_remote");

        assert!(
            !policy_remote.check(&test_context, Reason::Response),
            "count=8 < 10 should fail"
        );

        // For "agg_location", count that barely is big enough
        let row_location = vec![Value::from(0u64), Value::from(10u64)];
        let mut policy_location = GeneralKAnonPolicy::new();
        policy_location
            .initialize_from_row("agg_location", &row_location)
            .expect("Failed to initialize from agg_location");

        assert!(
            policy_location.check(&test_context, Reason::Response),
            "count=10 == 10 should pass"
        );

        // Test joining policies:
        // Joining a policy with count=12 and another with count=8 should result in min(12,8)=8
        // Should fail
        let joined = policy_gender.join(AnyPolicy::new(policy_remote)).unwrap();
        let joined = joined.specialize::<GeneralKAnonPolicy>().expect("Should be same type");
        assert!(
            !joined.check(&test_context, Reason::Response),
            "Joined count should be min(12,8)=8, failing check"
        );

        // Join a passing policy (count=10) with a failing policy (count=8):
        // The result should be min(10,8)=8 should fail
        let joined2 = policy_location.join(AnyPolicy::new(joined)).unwrap();
        let joined2 = joined2.specialize::<GeneralKAnonPolicy>().unwrap();
        assert!(
            !joined2.check(&test_context, Reason::Response),
            "count should still be 8"
        );

        // Now test joining two passing policies:
        let row_gender_high = vec![Value::from(1u64), Value::from(20u64)];
        let mut policy_gender_high = GeneralKAnonPolicy::new();
        policy_gender_high
            .initialize_from_row("agg_gender", &row_gender_high)
            .expect("Failed to initialize from agg_gender_high");
        assert!(
            policy_gender_high.check(&test_context, Reason::Response),
            "count=20 > 10 should pass"
        );

        // Join a passing policy (20) with another passing policy (12):
        let joined_passing = policy_gender_high
            .join(AnyPolicy::new(policy_gender))
            .expect("Join should succeed");
        let joined_passing = joined_passing.specialize::<GeneralKAnonPolicy>().unwrap();
        // After joining, count should be min(20,12)=12, still passing.
        assert!(
            joined_passing.check(&test_context, Reason::Response),
            "Count=12 should still pass"
        );

        // Validate schema correctness:
        assert!(
            joined_passing.validate_schema("agg_gender", 1),
            "'agg_gender' with column 1 is defined"
        );
        assert!(
            joined_passing.validate_schema("agg_remote", 1),
            "'agg_remote' with column 1 is defined"
        );
        assert!(
            joined_passing.validate_schema("agg_location", 1),
            "'agg_location' with column 1 is defined"
        );
        assert!(
            !joined_passing.validate_schema("agg_gender", 0),
            "Column mismatch should fail"
        );
        assert!(
            !joined_passing.validate_schema("unknown_table", 1),
            "Unknown table should fail"
        );

        // Test an invalid initialization:
        let row_invalid = vec![Value::from(0u64)]; // Only one column
        let mut policy_invalid = GeneralKAnonPolicy::new();
        let err = policy_invalid.initialize_from_row("agg_location", &row_invalid);
        assert!(
            err.is_err(),
            "Should fail because column index 1 is out of bounds"
        );
    }

    #[test]
    fn test_custom_schema_policy() {
        // Customers mapped to col 2
        let row = vec![
            mysql::Value::from("irrelevant"),
            mysql::Value::from(123u64),
            mysql::Value::from(999u64), // The one we want to count
        ];

        let context = UnprotectedContext {
            route: "/test".to_string(),
            data: Box::new(None::<()>),
        };

        let policy = CustomKAnonymityPolicy::from_row("customers", &row);
        assert_eq!(policy.count, 999, "Expected count to come from column 2");
        assert!(policy.check(&context, Reason::Response), "999 >= 5 should pass");
    }
}