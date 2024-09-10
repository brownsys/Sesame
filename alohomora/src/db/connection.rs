// BBox
use crate::{AlohomoraType, AlohomoraTypeEnum};
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

#[derive(Clone)]
pub struct BBoxStatement(Option<mysql::Statement>, String);
impl<'i> From<&'i str> for BBoxStatement {
    fn from(value: &'i str) -> Self {
        BBoxStatement(None, String::from(value))
    }
}
impl From<String> for BBoxStatement {
    fn from(value: String) -> Self {
        BBoxStatement(None, value)
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
    pub fn prep(&mut self, query: &str) -> BBoxResult<BBoxStatement> {
        let statement = self.conn.prep(query)?;
        Ok(BBoxStatement(Some(statement), String::from(query)))
    }

    // Text query and drop result.
    pub fn query_drop<T: AsRef<str>>(&mut self, query: T) -> BBoxResult<()> {
        self.conn.query_drop(query)
    }

    // Parameterized query and drop result.
    pub fn exec_drop<S: Into<BBoxStatement>, P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        stmt: S,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<()> {
        let stmt = stmt.into();
        let (statement, stmt_str) = (stmt.0, stmt.1);
        let statement = match statement {
            Some(statement) => statement,
            None => self.conn.prep(&stmt_str)?,
        };

        let params = params.into();
        let params = params.clone().transform(context, Reason::DB(&stmt_str, params.to_reason()))?;
        self.conn.exec_drop(statement, params)
    }

    // Parameterized query and return iterator to result.
    pub fn exec_iter<'i, S: Into<BBoxStatement>, P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        stmt: S,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<BBoxQueryResult<'_, '_, '_>> {
        let stmt = stmt.into();
        let (statement, stmt_str) = (stmt.0, stmt.1);
        let statement = match statement {
            Some(statement) => statement,
            None => self.conn.prep(&stmt_str)?,
        };

        let params = params.into();
        let params = params.clone().transform(context, Reason::DB(&stmt_str, params.to_reason()))?;
        let result = self.conn.exec_iter(statement, params)?;
        Ok(BBoxQueryResult { result })
    }

    // Chained prep and exec function
    pub fn prep_exec_drop<P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        query: &str,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<()> {
        let stmt = self.prep(query)?;
        self.exec_drop(stmt, params, context)
    }
    pub fn prep_exec_iter<P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        query: &str,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<BBoxQueryResult<'_, '_, '_>> {
        let stmt = self.prep(query)?;
        self.exec_iter(stmt, params, context)
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl AlohomoraType for BBoxConn {
    type Out = mysql::Conn;
    fn to_enum(self) -> AlohomoraTypeEnum {
      AlohomoraTypeEnum::Value(Box::new(self))
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Value(db) => {
                match db.downcast::<BBoxConn>() {
                  Ok(db) => Ok(db.conn),
                  Err(_) => Err(()),
                }
            }
            _ => Err(()),
        }
    } 
}
