use crate::db::{BBoxOpts, BBoxResult};
use crate::k9db::db::K9dbBBoxConn;
use crate::k9db::schema::table::Table;

pub struct K9db {
    conn_opts: BBoxOpts,
    tables: Vec<Table>,
}

impl K9db {
    pub fn new<T: Into<BBoxOpts>>(path: &str, opts: T) -> std::io::Result<K9db> {
        let f = std::fs::read_to_string(path)?;
        let tables = serde_json::de::from_str(&f)?;
        Ok(Self {
            conn_opts: opts.into(),
            tables,
        })
    }

    pub fn make_connection(&self) -> BBoxResult<K9dbBBoxConn> {
        K9dbBBoxConn::new(self.conn_opts.clone())
    }

    pub fn prime(&self) -> BBoxResult<()> {
        let mut conn = self.make_connection()?;
        conn.query_drop("SET echo;")?;
        for table in &self.tables {
            for sql in table.to_sql() {
                conn.query_drop(sql)?;
            }
        }
        Ok(())
    }

    pub fn declare_policies(&self) -> BBoxResult<()> {
        let mut conn = self.make_connection()?;
        conn.query_drop("SET echo;")?;
        for table in &self.tables {
            for sql in table.to_sql().into_iter().skip(1) {
                conn.query_drop(sql)?;
            }
        }
        Ok(())
    }

    pub fn to_sql(&self) -> Vec<String> {
        self.tables.iter().map(Table::to_sql).flatten().collect()
    }
}
