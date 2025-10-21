// BBox
use sesame::bbox::BBox;
use sesame::policy::AnyPolicy;

use crate::policy::get_schema_policies;
use crate::{BBoxFromValue, BBoxValue};

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

    pub fn get<T: BBoxFromValue, I: BBoxColumnIndex>(
        &self,
        index: I,
    ) -> Option<BBox<T, AnyPolicy>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns)?;
        let table = columns[idx].table_str().into_owned();
        let val = self.row.get(index)?;
        Some(BBox::new(val, get_schema_policies(table, idx, &self.raw)))
    }

    pub fn take<T: BBoxFromValue, I: BBoxColumnIndex>(
        &mut self,
        index: I,
    ) -> Option<BBox<T, AnyPolicy>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns)?;
        let table = columns[idx].table_str().into_owned();
        let val = self.row.take(index)?;
        Some(BBox::new(val, get_schema_policies(table, idx, &self.raw)))
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
