use std::collections::HashMap;
use std::sync::RwLock;

use crate::policy::{AnyPolicy, NoPolicy, PolicyAnd, SchemaPolicy};

// Global static singleton.
type SchemaPolicyFactory = dyn (Fn(&Vec<mysql::Value>) -> AnyPolicy) + Send + Sync;
type SchemaPolicyMap = HashMap<(String, usize), Vec<Box<SchemaPolicyFactory>>>;
lazy_static! {
    static ref SCHEMA_POLICIES: RwLock<SchemaPolicyMap> = RwLock::new(SchemaPolicyMap::new());
}

// Helper to fold an iterator of policies into an AndPolicy.
fn fold_policies<I: Iterator<Item = AnyPolicy>>(mut policies: I) -> AnyPolicy {
    match policies.next() {
        None => AnyPolicy::new(NoPolicy {}),
        Some(mut policy) => {
            for next in policies {
                policy = AnyPolicy::new(PolicyAnd::new(policy, next));
            }
            policy
        }
    }
}

// Create policies for a cell given its entire row and the name of its table.
pub fn get_schema_policies(
    table_name: String,
    column: usize,
    row: &Vec<mysql::Value>,
) -> AnyPolicy {
    let map = SCHEMA_POLICIES.read().unwrap();
    match (*map).get(&(table_name, column)) {
        Option::None => AnyPolicy::new(NoPolicy {}),
        Option::Some(factories) => fold_policies(factories.iter().map(|factory| factory(row))),
    }
}

// Register Policy T as a schema policy associated with the table and column.
// Never use this function directly, instead use the #[schema_policy(...)] macro.
extern crate small_ctor;
pub use small_ctor::ctor as register;
pub fn add_schema_policy<T: SchemaPolicy + Clone + 'static>(table_name: String, column: usize) {
    let mut map = SCHEMA_POLICIES.write().unwrap();
    map.entry((table_name.clone(), column))
        .or_default()
        .push(Box::new(move |row: &Vec<mysql::Value>| {
            AnyPolicy::new(T::from_row(&table_name, row))
        }));
}
