extern crate mysql;

// BBox
use crate::bbox::{BBox, EitherBBox};
use crate::policy::get_schema_policies;

// mysql imports.
pub use crate::db::mysql::prelude::Queryable;
pub use mysql::prelude::{AsStatement, ColumnIndex, FromValue};
pub use mysql::{Binary, Opts, Result, SetColumns, Statement};

// What is a (return) value.
pub type Value = BBox<mysql::Value>;

// Type modification.
pub fn from_value<T: FromValue>(v: Value) -> BBox<T> {
    v.into_map(|t| mysql::from_value(t))
}
pub fn from_value_or_null<T: FromValue>(v: Value) -> BBox<Option<T>> {
    v.into_map(|t| {
        if let mysql::Value::NULL = t {
            Option::None
        } else {
            mysql::from_value(t)
        }
    })
}

// Our params may be boxed or clear.
#[derive(Clone)]
pub struct Param(EitherBBox<mysql::Value>);

// Auto convert mysql::Value and bbox to Value.
impl<T: Into<mysql::Value>> From<T> for Param {
    fn from(x: T) -> Param {
        Param(EitherBBox::Value(x.into()))
    }
}
impl<T: Into<mysql::Value>> From<BBox<T>> for Param {
    fn from(x: BBox<T>) -> Param {
        Param(EitherBBox::BBox(x.into2()))
    }
}
impl<T: Into<mysql::Value> + Clone> From<&BBox<T>> for Param {
    fn from(x: &BBox<T>) -> Param {
        Param(EitherBBox::BBox(x.clone().into2()))
    }
}

// Our params could be mixed boxed and clear.
pub enum Params {
    Empty,
    // Named(HashMap<String, Value>),
    Positional(Vec<Param>),
}

// Can make Params from empty and Vec.
impl From<()> for Params {
    fn from(_: ()) -> Params {
        Params::Empty
    }
}
impl<T: Into<Param>> From<Vec<T>> for Params {
    fn from(x: Vec<T>) -> Params {
        let mut raw_params: Vec<Param> = Vec::new();
        for v in x.into_iter() {
            raw_params.push(v.into());
        }
        if raw_params.is_empty() {
            Params::Empty
        } else {
            Params::Positional(raw_params)
        }
    }
}

// Can make params from inlined function arguments.
macro_rules! into_params_impl {
  ($([$A:ident,$a:ident]),*) => (
    impl<$($A: Into<Param>,)*> From<($($A,)*)> for Params {
      fn from(x: ($($A,)*)) -> Params {
        let ($($a,)*) = x;
        Params::Positional(vec![
          $($a.into(),)*
        ])
      }
    }
  );
}
into_params_impl!([A, a]);
into_params_impl!([A, a], [B, b]);
into_params_impl!([A, a], [B, b], [C, c]);
into_params_impl!([A, a], [B, b], [C, c], [D, d]);
into_params_impl!([A, a], [B, b], [C, c], [D, d], [E, e]);
into_params_impl!([A, a], [B, b], [C, c], [D, d], [E, e], [F, f]);
into_params_impl!([A, a], [B, b], [C, c], [D, d], [E, e], [F, f], [G, g]);
into_params_impl!(
    [A, a],
    [B, b],
    [C, c],
    [D, d],
    [E, e],
    [F, f],
    [G, g],
    [H, h]
);

// A result row.
pub struct Row {
    row: mysql::Row,
    raw: Vec<mysql::Value>,
}
impl Row {
    pub fn new(row: mysql::Row) -> Self {
        let raw = row.clone().unwrap();
        Row { row, raw }
    }
    pub fn get<T: FromValue, I: ColumnIndex>(&self, index: I) -> Option<BBox<T>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns);
        match self.row.get(index) {
            Option::None => Option::None,
            Option::Some(t) => {
                let idx = idx.unwrap();
                let table = columns[idx].table_str().into_owned();
                Option::Some(BBox::new(t, get_schema_policies(table, idx, &self.raw)))
            }
        }
    }

    pub fn unwrap(self) -> Vec<Value> {
        self.raw
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let columns = self.row.columns_ref();
                let table = columns[i].table_str().into_owned();
                BBox::new(v.clone(), get_schema_policies(table, i, &self.raw))
            })
            .collect()
    }
}

// Our result wrapper.
pub struct QueryResult<'c, 't, 'tc> {
    result: mysql::QueryResult<'c, 't, 'tc, Binary>,
}
impl<'c, 't, 'tc> QueryResult<'c, 't, 'tc> {
    pub fn affected_rows(&self) -> u64 {
        self.result.affected_rows()
    }
    pub fn last_insert_id(&self) -> Option<u64> {
        self.result.last_insert_id()
    }
    pub fn columns(&self) -> SetColumns<'_> {
        self.result.columns()
    }
}
impl<'c, 't, 'tc> Iterator for QueryResult<'c, 't, 'tc> {
    type Item = Result<Row>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.result.next() {
            None => None,
            Some(row) => match row {
                Ok(row) => Some(Ok(Row::new(row))),
                Err(e) => Some(Err(e)),
            },
        }
    }
}

// BBox DB connection
pub struct Conn {
    conn: mysql::Conn,
}

impl Conn {
    // Creating a new DBConn is the same as creating a new mysql::Conn.
    pub fn new<T: Into<Opts>>(opts: T) -> Result<Conn> {
        let conn = mysql::Conn::new(opts)?;
        Ok(Conn { conn: conn })
    }

    // Test ping.
    pub fn ping(&mut self) -> bool {
        self.conn.ping()
    }

    // Prepare a statement.
    pub fn prep<T: AsRef<str>>(&mut self, query: T) -> Result<Statement> {
        self.conn.prep(query)
    }

    // Text query and drop result.
    pub fn query_drop<T: AsRef<str>>(&mut self, query: T) -> Result<()> {
        self.conn.query_drop(query)
    }

    // Parameterized query and drop result.
    pub fn exec_drop<S: AsStatement, P: Into<Params>>(&mut self, stmt: S, params: P) -> Result<()> {
        let params = self.unbox_params(params.into());
        self.conn.exec_drop(stmt, params)
    }

    // Parameterized query and return iterator to result.
    pub fn exec_iter<S: AsStatement, P: Into<Params>>(
        &mut self,
        stmt: S,
        params: P,
    ) -> Result<QueryResult<'_, '_, '_>> {
        let params = self.unbox_params(params.into());
        let result = self.conn.exec_iter(stmt, params)?;
        Ok(QueryResult { result })
    }

    // private helper function.
    fn unbox_params(&self, params: Params) -> mysql::params::Params {
        match params {
            Params::Empty => mysql::params::Params::Empty,
            Params::Positional(vec) => {
                let unboxed = vec
                    .into_iter()
                    .map(|v: Param| match v {
                        Param(EitherBBox::Value(v)) => v,
                        Param(EitherBBox::BBox(bbox)) => bbox.t,
                    })
                    .collect();
                mysql::params::Params::Positional(unboxed)
            }
        }
    }
}
