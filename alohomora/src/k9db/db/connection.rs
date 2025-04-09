use crate::{AlohomoraType, AlohomoraTypeEnum};
use crate::context::{Context, ContextData};
use crate::db::{BBoxConn, BBoxOpts, BBoxParams, BBoxResult, BBoxStatement};
use crate::k9db::db::BBoxK9dbQueryResult;

pub struct K9dbBBoxConn {
    conn: BBoxConn,
}

impl K9dbBBoxConn {
    // Creating a new DBConn is the same as creating a new mysql::Conn.
    pub fn new<T: Into<BBoxOpts>>(opts: T) -> BBoxResult<K9dbBBoxConn> {
        Ok(K9dbBBoxConn { conn: BBoxConn::new(opts)? })
    }

    // Test ping.
    pub fn ping(&mut self) -> bool {
        self.conn.ping()
    }

    // Prepare a statement.
    pub fn prep(&mut self, query: &str) -> BBoxResult<BBoxStatement> {
        self.conn.prep(query)
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
        self.conn.exec_drop(stmt, params, context)
    }

    pub fn query_iter<T: AsRef<str>>(
        &mut self,
        query: T
    ) -> BBoxResult<BBoxK9dbQueryResult<'_, '_, '_, mysql::Text>> {
        let query_result = self.conn.query_iter(query)?;
        Ok(BBoxK9dbQueryResult::new(query_result.result))
    }

    // Parameterized query and return iterator to result.
    pub fn exec_iter<'i, S: Into<BBoxStatement>, P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        stmt: S,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<BBoxK9dbQueryResult<'_, '_, '_, mysql::Binary>> {
        let query_result = self.conn.exec_iter(stmt, params, context)?;
        Ok(BBoxK9dbQueryResult::new(query_result.result))
    }

    // Chained prep and exec function
    pub fn prep_exec_drop<P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        query: &str,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<()> {
        self.conn.prep_exec_drop(query, params, context)
    }
    pub fn prep_exec_iter<P: Into<BBoxParams>, D: ContextData>(
        &mut self,
        query: &str,
        params: P,
        context: Context<D>,
    ) -> BBoxResult<BBoxK9dbQueryResult<'_, '_, '_, mysql::Binary>> {
        let query_result = self.conn.prep_exec_iter(query, params, context)?;
        Ok(BBoxK9dbQueryResult::new(query_result.result))
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl AlohomoraType for K9dbBBoxConn {
    type Out = <BBoxConn as AlohomoraType>::Out;
    fn to_enum(self) -> AlohomoraTypeEnum {
        self.conn.to_enum()
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        BBoxConn::from_enum(e)
    }
}