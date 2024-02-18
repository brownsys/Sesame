// BBox
use crate::db::{BBoxParams, BBoxQueryResult, unbox_params};

// mysql imports.
use mysql::prelude::Queryable;
pub use mysql::prelude::AsStatement as BBoxAsStatement;
pub use mysql::{Opts as BBoxOpts, Statement as BBoxStatement};
pub use mysql::Result as BBoxResult;

// BBox DB connection
pub struct BBoxConn {
    conn: mysql::Conn,
}

impl BBoxConn {
    // Creating a new DBConn is the same as creating a new mysql::Conn.
    pub fn new<T: Into<BBoxOpts>>(opts: T) -> BBoxResult<BBoxConn> {
        Ok(BBoxConn { conn: mysql::Conn::new(opts)? })
    }

    // Test ping.
    pub fn ping(&mut self) -> bool {
        self.conn.ping()
    }

    // Prepare a statement.
    pub fn prep<T: AsRef<str>>(&mut self, query: T) -> BBoxResult<BBoxStatement> {
        self.conn.prep(query)
    }

    // Text query and drop result.
    pub fn query_drop<T: AsRef<str>>(&mut self, query: T) -> BBoxResult<()> {
        self.conn.query_drop(query)
    }

    // Parameterized query and drop result.
    pub fn exec_drop<S: BBoxAsStatement, P: Into<BBoxParams>>(
        &mut self,
        stmt: S,
        params: P,
    ) -> BBoxResult<()> {
        let params = unbox_params(params.into());
        self.conn.exec_drop(stmt, params)
    }

    // Parameterized query and return iterator to result.
    pub fn exec_iter<S: BBoxAsStatement, P: Into<BBoxParams>>(
        &mut self,
        stmt: S,
        params: P,
    ) -> BBoxResult<BBoxQueryResult<'_, '_, '_>> {
        let params = unbox_params(params.into());
        let result = self.conn.exec_iter(stmt, params)?;
        Ok(BBoxQueryResult { result })
    }
}
