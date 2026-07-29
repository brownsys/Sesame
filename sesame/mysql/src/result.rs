use std::sync::Arc;

use crate::policy::ColumnPolicies;
use crate::PConRow;

// mysql imports.
pub use mysql::SetColumns as PConSetColumns;

// Our result wrapper. Resolves the schema-policy factories for its
// column set once at construction; every row shares that snapshot, so
// per-cell policy attachment never touches the global registry.
pub struct PConQueryResult<'c, 't, 'tc, T: mysql::prelude::Protocol> {
    pub(crate) result: mysql::QueryResult<'c, 't, 'tc, T>,
    policies: Arc<ColumnPolicies>,
}
impl<'c, 't, 'tc, T: mysql::prelude::Protocol> PConQueryResult<'c, 't, 'tc, T> {
    pub(crate) fn new(result: mysql::QueryResult<'c, 't, 'tc, T>) -> Self {
        let policies = ColumnPolicies::resolve(result.columns().as_ref());
        PConQueryResult { result, policies }
    }
    pub fn affected_rows(&self) -> u64 {
        self.result.affected_rows()
    }
    pub fn last_insert_id(&self) -> Option<u64> {
        self.result.last_insert_id()
    }
    pub fn columns(&self) -> PConSetColumns<'_> {
        self.result.columns()
    }
}
impl<'c, 't, 'tc, T: mysql::prelude::Protocol> Iterator for PConQueryResult<'c, 't, 'tc, T> {
    type Item = mysql::Result<PConRow>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.result.next() {
            None => None,
            Some(row) => match row {
                Ok(row) => Some(Ok(PConRow::new(row, Arc::clone(&self.policies)))),
                Err(e) => Some(Err(e)),
            },
        }
    }
}
