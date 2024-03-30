use std::borrow::Cow;
use std::ops::Deref;
// BBox
use crate::db::{BBoxParams, BBoxQueryResult};

// mysql imports.
use mysql::prelude::Queryable;
pub use mysql::{Opts as BBoxOpts};
pub use mysql::Result as BBoxResult;
use crate::context::{Context, ContextData};
use crate::policy::Reason;

// BBox DB connection
pub struct BBoxConn {
    conn: mysql::Conn,
}

pub struct BBoxStatement<'i>(Option<mysql::Statement>, Cow<'i, str>);
impl<'i> From<&'i str> for BBoxStatement<'i> {
    fn from(value: &'i str) -> Self {
        BBoxStatement(None, Cow::Borrowed(value))
    }
}
impl<'i> From<String> for BBoxStatement<'i> {
    fn from(value: String) -> Self {
        BBoxStatement(None, Cow::Owned(value))
    }
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
    pub fn prep<'i, T: Into<Cow<'i, str>>>(&mut self, query: T) -> BBoxResult<BBoxStatement<'i>> {
        let query = query.into();
        let statement = self.conn.prep(query.deref())?;
        Ok(BBoxStatement(Some(statement), query))
    }

    // Text query and drop result.
    pub fn query_drop<T: AsRef<str>>(&mut self, query: T) -> BBoxResult<()> {
        self.conn.query_drop(query)
    }

    // Parameterized query and drop result.
    pub fn exec_drop<'i, S: Into<BBoxStatement<'i>>, P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        stmt: S,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<()> {
        let stmt = stmt.into();
        let (statement, stmt_str) = (stmt.0, stmt.1);
        let statement = match statement {
            Some(statement) => statement,
            None => self.conn.prep(stmt_str.deref())?,
        };

        let params = params.into().transform(context, Reason::DB(stmt_str.deref()))?;
        self.conn.exec_drop(statement, params)
    }

    // Parameterized query and return iterator to result.
    pub fn exec_iter<'i, S: Into<BBoxStatement<'i>>, P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        stmt: S,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<BBoxQueryResult<'_, '_, '_>> {
        let stmt = stmt.into();
        let (statement, stmt_str) = (stmt.0, stmt.1);
        let statement = match statement {
            Some(statement) => statement,
            None => self.conn.prep(stmt_str.deref())?,
        };

        let params = params.into().transform(context, Reason::DB(stmt_str.deref()))?;
        let result = self.conn.exec_iter(statement, params)?;
        Ok(BBoxQueryResult { result })
    }

    // Chained prep and exec function
    pub fn prep_exec_drop<'i, T: Into<Cow<'i, str>>, P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        query: T,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<()> {
        let stmt = self.prep(query)?;
        self.exec_drop(stmt, params, context)
    }
    pub fn prep_exec_iter<'i, T: Into<Cow<'i, str>>, P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        query: T,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<BBoxQueryResult<'_, '_, '_>> {
        let stmt = self.prep(query)?;
        self.exec_iter(stmt, params, context)
    }
}