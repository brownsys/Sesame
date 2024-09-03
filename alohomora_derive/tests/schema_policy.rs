use alohomora::policy::{AnyPolicy, NoPolicy, Policy, Reason, SchemaPolicy};
use alohomora_derive::schema_policy;

use mysql::Value;
use alohomora::context::UnprotectedContext;

#[schema_policy(table = "my_table", column = 3)]
#[derive(Clone)]
pub struct SamplePolicy {}
impl Policy for SamplePolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &UnprotectedContext, _: Reason) -> bool {
        true
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl SchemaPolicy for SamplePolicy {
    fn from_row(_table: &str, _row: &Vec<Value>) -> Self {
        SamplePolicy {}
    }
}

#[test]
fn schema_policy_registration_test() {
    let policy = alohomora::policy::get_schema_policies(String::from("my_table"), 3, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(SamplePolicy)"));
    assert!(policy.check(&UnprotectedContext::test(()), Reason::Custom(Box::new(()))));
    assert!(policy.is::<SamplePolicy>());
    let policy: SamplePolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("SamplePolicy"));

    let policy = alohomora::policy::get_schema_policies(String::from("my_table"), 2, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(NoPolicy)"));
    assert!(policy.check(&UnprotectedContext::test(()), Reason::Custom(Box::new(()))));
    assert!(policy.is::<NoPolicy>());
    let policy: NoPolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("NoPolicy"));

    let policy = alohomora::policy::get_schema_policies(String::from("table"), 3, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(NoPolicy)"));
    assert!(policy.check(&UnprotectedContext::test(()), Reason::Custom(Box::new(()))));
    assert!(policy.is::<NoPolicy>());
    let policy: NoPolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("NoPolicy"));
}
