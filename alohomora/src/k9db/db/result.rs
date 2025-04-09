use crate::db::{BBoxResult};
use crate::k9db::db::{BBoxK9dbColumnSet, BBoxK9dbRow};


// Our result wrapper.
pub struct BBoxK9dbQueryResult<'c, 't, 'tc, T: mysql::prelude::Protocol> {
    result: mysql::QueryResult<'c, 't, 'tc, T>,
    columns: BBoxK9dbColumnSet,
}
impl<'c, 't, 'tc, T: mysql::prelude::Protocol> BBoxK9dbQueryResult<'c, 't, 'tc, T> {
    pub fn new(result: mysql::QueryResult<'c, 't, 'tc, T>) -> Self {
        let columns = BBoxK9dbColumnSet::new(result.columns());
        Self { result, columns }
    }
    pub fn affected_rows(&self) -> u64 {
        self.result.affected_rows()
    }
    pub fn last_insert_id(&self) -> Option<u64> {
        self.result.last_insert_id()
    }
    pub fn columns(&self) -> &BBoxK9dbColumnSet {
        &self.columns
    }
}
impl<'c, 't, 'tc, T: mysql::prelude::Protocol> Iterator for BBoxK9dbQueryResult<'c, 't, 'tc, T> {
    type Item = BBoxResult<BBoxK9dbRow>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.result.next()? {
            Ok(row) => Some(Ok(BBoxK9dbRow::new(row.unwrap(), &self.columns))),
            Err(e) => Some(Err(e)),
        }
    }
}