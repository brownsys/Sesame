extern crate mysql;

// BBox
use crate::{BBox, VBox};
use crate::policy::get_schema_policies;

// mysql imports.
pub use mysql::{Opts, Statement, Result, Binary, SetColumns};
pub use mysql::prelude::{AsStatement, ColumnIndex, FromValue};
pub use crate::db::mysql::prelude::Queryable;

// What is a (return) value.
pub type Value = BBox<mysql::Value>;

// Type modification.
pub fn from_value<T: FromValue>(v: Value) -> BBox<T> {
  BBox::new_with_policy(mysql::from_value(v.t), v.policies.clone())
}
pub fn from_value_or_null<T: FromValue>(v: Value) -> BBox<Option<T>> {
  if let mysql::Value::NULL = v.t {
    BBox::new_with_policy(None, v.policies.clone())
  } else {
    BBox::new_with_policy(Some(mysql::from_value(v.t)), v.policies.clone())
  }
}

// Our params may be boxed or clear.
#[derive(Clone)]
pub struct Param (VBox<mysql::Value>);

// Auto convert mysql::Value and bbox to Value.
impl<T: Into<mysql::Value>> From<T> for Param {
  fn from(x: T) -> Param {
    Param(VBox::Value(x.into()))
  }
}
impl<T: Into<mysql::Value>> From<BBox<T>> for Param {
  fn from(x: BBox<T>) -> Param {
    Param(VBox::BBox(x.m_into2()))
  }
}
impl<T: Into<mysql::Value> + Clone> From<&BBox<T>> for Param {
  fn from(x: &BBox<T>) -> Param {
    Param(VBox::BBox(x.into2()))
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
into_params_impl!([A, a], [B, b], [C, c], [D, d], [E, e], [F, f], [G, g], [H, h]);

// A result row.
pub struct Row {
  row: mysql::Row,
  table: String,
}
impl Row {
  pub fn get<T: FromValue, I: ColumnIndex>(&self, index: I) -> Option<BBox<T>> {
    let idx = index.idx(self.row.columns_ref()).unwrap();
    let row = self.row.clone().unwrap();
    match self.row.get(index) {
      Option::None => Option::None,
      Option::Some(t) => {
        Option::Some(BBox::new_with_policy(t, get_schema_policies(self.table.clone(), idx, &row)))
      },
    }
  }

  pub fn unwrap(self) -> Vec<Value> {
    let table = self.table.clone();
    let vals = self.row.unwrap();
    vals.iter().enumerate().map(|(i, v)| BBox::new_with_policy(v.clone(), get_schema_policies(table.clone(), i, &vals))).collect()
  }
}

// Our result wrapper.
pub struct QueryResult<'c, 't, 'tc> {
  result: mysql::QueryResult<'c, 't, 'tc, Binary>,
  table: String,
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
      Some(row) => {
        match row {
          Ok(row) => Some(Ok(Row { row: row, table: self.table.clone() })),
          Err(e) => Some(Err(e))
        }
      }
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
  pub fn exec_drop<S: AsStatement, P: Into<Params>>(
      &mut self, stmt: S, params: P) -> Result<()> {
    let params = self.unbox_params(params.into());
    self.conn.exec_drop(stmt, params)
  }

  // Parameterized query and return iterator to result.
  pub fn exec_iter<S: AsStatement, P: Into<Params>>(
      &mut self, stmt: S, params: P) -> Result<QueryResult<'_, '_, '_>> {
    let params = self.unbox_params(params.into());
    let result = self.conn.exec_iter(stmt, params)?;
    let columns = result.columns();
    // TODO(artem): should find a better way to extract table name
    let table: String = columns.as_ref()[0].table_str().parse()?;
    Ok(QueryResult { result, table })
  }
  
  // private helper function.
  fn unbox_params(&self, params: Params) -> mysql::params::Params {
    match params {
      Params::Empty => mysql::params::Params::Empty,
      Params::Positional(vec) => {
        let unboxed = vec.into_iter().map(|v: Param| {
          match v {
            Param(VBox::Value(v)) => v,
            Param(VBox::BBox(bbox)) => bbox.t,
          }
        }).collect();
        mysql::params::Params::Positional(unboxed)
      },
    }
  }
}
