use sesame::context::{Context, ContextData};
use sesame::policy::Reason;
use sesame::SesameTypeOut;
use sesame::{SesameType, SesameTypeEnum};

// mysql imports.
use mysql::prelude::Queryable;
pub use mysql::Opts as PConOpts;

use crate::{PConParams, PConQueryResult, PConResult};

// PCon DB connection
pub struct SesameConn {
    conn: mysql::Conn,
}

#[derive(Clone)]
pub struct PConStatement(Option<mysql::Statement>, String);
impl<'i> From<&'i str> for PConStatement {
    fn from(value: &'i str) -> Self {
        PConStatement(None, String::from(value))
    }
}
impl From<String> for PConStatement {
    fn from(value: String) -> Self {
        PConStatement(None, value)
    }
}

impl SesameConn {
    // Creating a new DBConn is the same as creating a new mysql::Conn.
    pub fn new<T: Into<PConOpts>>(opts: T) -> PConResult<SesameConn> {
        Ok(SesameConn {
            conn: mysql::Conn::new(opts)?,
        })
    }

    // Test ping.
    pub fn ping(&mut self) -> bool {
        self.conn.ping()
    }

    // Prepare a statement.
    pub fn prep(&mut self, query: &str) -> PConResult<PConStatement> {
        let statement = self.conn.prep(query)?;
        Ok(PConStatement(Some(statement), String::from(query)))
    }

    // Text query and drop result.
    pub fn query_drop<T: AsRef<str>>(&mut self, query: T) -> PConResult<()> {
        Ok(self.conn.query_drop(query)?)
    }

    // Parameterized query and drop result.
    pub fn exec_drop<S: Into<PConStatement>, P: Into<PConParams>, D: ContextData>(
        &mut self,
        stmt: S,
        params: P,
        context: Context<D>,
    ) -> PConResult<()> {
        let stmt = stmt.into();
        let (statement, stmt_str) = (stmt.0, stmt.1);
        let statement = match statement {
            Some(statement) => statement,
            None => self.conn.prep(&stmt_str)?,
        };

        let params = params.into();
        let param_values = params.to_reason();
        let params = params.transform(
            context,
            Reason::DB(&stmt_str, param_values.iter().collect()),
        )?;
        Ok(self.conn.exec_drop(statement, params)?)
    }

    pub fn query_iter<T: AsRef<str>>(
        &mut self,
        query: T,
    ) -> PConResult<PConQueryResult<'_, '_, '_, mysql::Text>> {
        let result = self.conn.query_iter(query)?;
        Ok(PConQueryResult { result })
    }

    // Parameterized query and return iterator to result.
    pub fn exec_iter<'i, S: Into<PConStatement>, P: Into<PConParams>, D: ContextData>(
        &mut self,
        stmt: S,
        params: P,
        context: Context<D>,
    ) -> PConResult<PConQueryResult<'_, '_, '_, mysql::Binary>> {
        let stmt = stmt.into();
        let (statement, stmt_str) = (stmt.0, stmt.1);
        let statement = match statement {
            Some(statement) => statement,
            None => self.conn.prep(&stmt_str)?,
        };

        let params = params.into();
        let param_values = params.to_reason();
        let params = params.transform(
            context,
            Reason::DB(&stmt_str, param_values.iter().collect()),
        )?;
        let result = self.conn.exec_iter(statement, params)?;
        Ok(PConQueryResult { result })
    }

    // Chained prep and exec function
    pub fn prep_exec_drop<P: Into<PConParams>, D: ContextData>(
        &mut self,
        query: &str,
        params: P,
        context: Context<D>,
    ) -> PConResult<()> {
        let stmt = self.prep(query)?;
        self.exec_drop(stmt, params, context)
    }
    pub fn prep_exec_iter<P: Into<PConParams>, D: ContextData>(
        &mut self,
        query: &str,
        params: P,
        context: Context<D>,
    ) -> PConResult<PConQueryResult<'_, '_, '_, mysql::Binary>> {
        let stmt = self.prep(query)?;
        self.exec_iter(stmt, params, context)
    }
}

#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl SesameTypeOut for SesameConn {
    type Out = mysql::Conn;
}

#[doc = "Library implementation of SesameType. Do not copy this docstring!"]
impl SesameType for SesameConn {
    fn to_enum(self) -> SesameTypeEnum {
        SesameTypeEnum::Value(Box::new(self))
    }
    fn from_enum(e: SesameTypeEnum) -> Result<Self, ()> {
        match e {
            SesameTypeEnum::Value(db) => match db.downcast::<SesameConn>() {
                Ok(db) => Ok(*db),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
    fn out_from_enum(e: SesameTypeEnum) -> Result<Self::Out, ()> {
        match e {
            SesameTypeEnum::Value(db) => match db.downcast::<SesameConn>() {
                Ok(db) => Ok(db.conn),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}
