use bbox_derive::schema_policy;

use bbox::policy::{AnyPolicy, NoPolicy, Policy, SchemaPolicy};

use mysql::Value;
use std::any::Any;

#[schema_policy(table = "my_table", column = 3)]
#[derive(Clone)]
pub struct SamplePolicy {}
impl Policy for SamplePolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &dyn Any) -> bool {
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
    fn from_row(_row: &Vec<Value>) -> Self {
        SamplePolicy {}
    }
}

#[test]
fn schema_policy_registration_test() {
    let policy = bbox::policy::get_schema_policies(String::from("my_table"), 3, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(SamplePolicy)"));
    assert!(policy.check(&""));
    assert!(policy.is::<SamplePolicy>());
    let policy: SamplePolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("SamplePolicy"));

    let policy = bbox::policy::get_schema_policies(String::from("my_table"), 2, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(NoPolicy)"));
    assert!(policy.check(&""));
    assert!(policy.is::<NoPolicy>());
    let policy: NoPolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("NoPolicy"));

    let policy = bbox::policy::get_schema_policies(String::from("table"), 3, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(NoPolicy)"));
    assert!(policy.check(&""));
    assert!(policy.is::<NoPolicy>());
    let policy: NoPolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("NoPolicy"));
}
