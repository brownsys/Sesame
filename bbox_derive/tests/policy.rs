use bbox_derive::schema_policy;

use bbox::policy::{NoPolicy, Policy, SchemaPolicy};

use std::any::Any;
use mysql::Value;

#[schema_policy(table = "my_table", column = 3)]
pub struct SamplePolicy {}
impl Policy for SamplePolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &dyn Any) -> bool {
        true
    }
}
impl SchemaPolicy for SamplePolicy {
    fn from_row(_row: &Vec<Value>) -> Self {
        SamplePolicy {}
    }
}

#[test]
fn simple_render_struct() {
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
