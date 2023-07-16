use bbox_derive::schema_policy;

use bbox::policy::Policy;

use std::any::Any;

#[schema_policy(table = "my_table", column = 3)]
pub struct SamplePolicy {}
impl Policy for SamplePolicy {
    fn from_row(_: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        SamplePolicy {}
    }
    fn check(&self, _: &dyn Any) -> bool {
        true
    }
}

#[test]
fn simple_render_struct() {
    let policies = bbox::policy::get_schema_policies(String::from("my_table"), 3, &vec![]);
    assert_eq!(policies.len(), 1);
    assert!(policies[0].lock().unwrap().check(&""));

    let policies = bbox::policy::get_schema_policies(String::from("my_table"), 2, &vec![]);
    assert_eq!(policies.len(), 0);

    let policies = bbox::policy::get_schema_policies(String::from("table"), 3, &vec![]);
    assert_eq!(policies.len(), 0);
}
