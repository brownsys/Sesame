use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

// TODO(babman): can we make this Any any cleaner?
pub trait Policy : Send {
    // fn from_http_request(...) -> Self;
    fn from_row(row: &Vec<mysql::Value>) -> Self where Self: Sized;
    fn check(&self, context: &dyn Any) -> bool;
}

// Global static singleton.
type SchemaPolicyFactory = dyn (Fn(&Vec<mysql::Value>) -> Arc<Mutex<dyn Policy>>) + Send + Sync;
type SchemaPolicyMap = HashMap<(String, usize), Vec<Box<SchemaPolicyFactory>>>;
lazy_static! {
  static ref SCHEMA_POLICIES: RwLock<SchemaPolicyMap> = RwLock::new(SchemaPolicyMap::new());
}

// Create policies for a cell given its entire row and the name of its table.
pub(crate) fn get_schema_policies(table_name: String, column: usize, row: &Vec<mysql::Value>) -> Vec<Arc<Mutex<dyn Policy>>> {
  let map = SCHEMA_POLICIES.read().unwrap();
  match (*map).get(&(table_name, column)) {
    Option::None => vec![],
    Option::Some(factories) => factories.iter().map(|factory| factory(row)).collect(),
  }
}

// Register Policy T as a schema policy associated with the table and column.
// Never use this function directly, instead use the #[schema_policy(...)] macro.
extern crate small_ctor;
pub use small_ctor::ctor as register;
pub fn add_schema_policy<T: Policy + 'static>(table_name: String, column: usize) { 
  let mut map = SCHEMA_POLICIES.write().unwrap();
  map.entry((table_name, column)).or_default().push(Box::new(|row: &Vec<mysql::Value>| {
    Arc::new(Mutex::new(T::from_row(row)))
  }));
}
