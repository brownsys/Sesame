// BBox
use crate::bbox::{BBox};
use crate::policy::{get_schema_policies, AnyPolicy};
use crate::db::{BBoxValue, BBoxFromValue};

// mysql imports.
pub use mysql::prelude::ColumnIndex as BBoxColumnIndex;

// A result row.
#[derive(Clone)]
pub struct BBoxRow {
    row: mysql::Row,
    raw: Vec<mysql::Value>,
}
impl BBoxRow {
    pub(super) fn new(row: mysql::Row) -> Self {
        let raw = row.clone().unwrap();
        BBoxRow { row, raw }
    }
    pub fn get<T: BBoxFromValue, I: BBoxColumnIndex>(&self, index: I) -> Option<BBox<T, AnyPolicy>> {
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

    pub fn unwrap(self) -> Vec<BBoxValue> {
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
