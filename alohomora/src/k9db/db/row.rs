use mysql::Column;

// BBox
use crate::bbox::{BBox};
use crate::policy::{AnyPolicy};
use crate::db::{BBoxValue, BBoxFromValue};
use crate::k9db::db::{attach_policies_to_values, BBoxK9dbColumnSet};

// mysql imports.
pub use mysql::prelude::ColumnIndex as BBoxColumnIndex;

// A result row.
#[derive(Clone)]
pub struct BBoxK9dbRow {
    row: Vec<BBoxValue>,
    columns: Vec<Column>,  // only value columns.
}
impl BBoxK9dbRow {
    pub fn new(row: Vec<mysql::Value>, columns: &BBoxK9dbColumnSet) -> BBoxK9dbRow {
        Self {
            row: attach_policies_to_values(row, columns),
            columns: columns.columns.clone(),
        }
    }
    pub fn get<T: BBoxFromValue, I: BBoxColumnIndex>(&self, index: I) -> Option<BBox<T, AnyPolicy>> {
        let idx = index.idx(&self.columns)?;
        match self.row.get(idx) {
            None => None,
            Some(bbox) => {
                let (t, p) = bbox.clone().consume();
                Some(BBox::new(mysql::from_value(t), p))
            }
        }
    }
    pub fn unwrap(self) -> Vec<BBoxValue> {
        self.row
    }
}
