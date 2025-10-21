use sesame::context::UnprotectedContext;
use sesame::policy::{NoPolicy, Policy, Reason, SimplePolicy, Specializable};
use sesame_derive::schema_policy;
use sesame_mysql::SchemaPolicy;

use mysql::Value;

#[schema_policy(table = "my_table", column = 3)]
#[derive(Clone)]
pub struct SamplePolicy {}
impl SimplePolicy for SamplePolicy {
    fn simple_name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn simple_check(&self, _: &UnprotectedContext, _: Reason) -> bool {
        true
    }
    fn simple_join_direct(&mut self, _other: &mut Self) {}
}
impl SchemaPolicy for SamplePolicy {
    fn from_row(_table: &str, _row: &Vec<Value>) -> Self {
        SamplePolicy {}
    }
}

#[test]
fn schema_policy_registration_test() {
    let policy = sesame_mysql::get_schema_policies(String::from("my_table"), 3, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(SamplePolicy)"));
    assert!(policy.check(&UnprotectedContext::test(()), Reason::Custom(&())));
    assert!(policy.is::<SamplePolicy>());
    let policy: SamplePolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("SamplePolicy"));

    let policy = sesame_mysql::get_schema_policies(String::from("my_table"), 2, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(NoPolicy)"));
    assert!(policy.check(&UnprotectedContext::test(()), Reason::Custom(&())));
    assert!(policy.is::<NoPolicy>());
    let policy: NoPolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("NoPolicy"));

    let policy = sesame_mysql::get_schema_policies(String::from("table"), 3, &vec![]);
    assert_eq!(policy.name(), String::from("AnyPolicy(NoPolicy)"));
    assert!(policy.check(&UnprotectedContext::test(()), Reason::Custom(&())));
    assert!(policy.is::<NoPolicy>());
    let policy: NoPolicy = policy.specialize().unwrap();
    assert_eq!(policy.name(), String::from("NoPolicy"));
}
