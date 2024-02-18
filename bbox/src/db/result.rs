// BBox
use crate::db::BBoxRow;

// mysql imports.
pub use mysql::SetColumns as BBoxSetColumns;

// Our result wrapper.
pub struct BBoxQueryResult<'c, 't, 'tc> {
    pub(super) result: mysql::QueryResult<'c, 't, 'tc, mysql::Binary>,
}
impl<'c, 't, 'tc> BBoxQueryResult<'c, 't, 'tc> {
    pub fn affected_rows(&self) -> u64 {
        self.result.affected_rows()
    }
    pub fn last_insert_id(&self) -> Option<u64> {
        self.result.last_insert_id()
    }
    pub fn columns(&self) -> BBoxSetColumns<'_> {
        self.result.columns()
    }
}
impl<'c, 't, 'tc> Iterator for BBoxQueryResult<'c, 't, 'tc> {
    type Item = mysql::Result<BBoxRow>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.result.next() {
            None => None,
            Some(row) => match row {
                Ok(row) => Some(Ok(BBoxRow::new(row))),
                Err(e) => Some(Err(e)),
            },
        }
    }
}
