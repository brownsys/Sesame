use std::collections::HashMap;

use std::any::Any;

use crate::context::Context;

// TODO(babman): can we make this Any any cleaner?
pub trait Policy : Send {
    // fn from_http_request(...) -> Self;
    //fn from_row(row: &Vec<mysql::Value>) -> Self where Self: Sized;
    fn check(&self, context: &dyn Any) -> bool;
}

pub trait PolicyFactory : Send {
    fn create(&self, row: &Vec<mysql::Value>) -> Box<dyn Policy>;
}


pub struct PolicyManager {
  map: HashMap<(String, usize), Vec<Box<dyn PolicyFactory>>>,
}

impl PolicyManager {
  pub fn new() -> Self {
    PolicyManager { map: HashMap::new() }
  }

  pub fn manage(&self, table_name: &str, column: usize, row: &Vec<mysql::Value>) -> Vec<Box<dyn Policy>> {
    match self.map.get(&(String::from(table_name), column)) {
      None => vec![],
      Some(factories) => factories.iter().map(|factory| factory.as_ref().create(row)).collect(),
    }
  }
  
  pub fn add<T: PolicyFactory + 'static>(&mut self, table_name: String, column: usize, factory: T) {
    self.map.entry((table_name, column)).or_default().push(Box::new(factory));
  }
}
