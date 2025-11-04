use crate::PConRow;

// mysql imports.
pub use mysql::SetColumns as PConSetColumns;

// Our result wrapper.
pub struct PConQueryResult<'c, 't, 'tc, T: mysql::prelude::Protocol> {
    pub(crate) result: mysql::QueryResult<'c, 't, 'tc, T>,
}
impl<'c, 't, 'tc, T: mysql::prelude::Protocol> PConQueryResult<'c, 't, 'tc, T> {
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
                Ok(row) => Some(Ok(PConRow::new(row))),
                Err(e) => Some(Err(e)),
            },
        }
    }
}
