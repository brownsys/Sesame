use sesame::pcon::PCon;
use sesame::policy::AnyPolicy;

use crate::policy::get_schema_policies;
use crate::{PConFromValue, PConValue};

// mysql imports.
pub use mysql::prelude::ColumnIndex as PConColumnIndex;

// A result row.
#[derive(Clone)]
pub struct PConRow {
    row: mysql::Row,
    raw: Vec<mysql::Value>,
}
impl PConRow {
    pub(super) fn new(row: mysql::Row) -> Self {
        let raw = row.clone().unwrap();
        PConRow { row, raw }
    }

    pub fn get<T: PConFromValue, I: PConColumnIndex>(
        &self,
        index: I,
    ) -> Option<PCon<T, AnyPolicy>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns)?;
        let table = columns[idx].table_str().into_owned();
        let val = self.row.get(index)?;
        Some(PCon::new(val, get_schema_policies(table, idx, &self.raw)))
    }

    pub fn take<T: PConFromValue, I: PConColumnIndex>(
        &mut self,
        index: I,
    ) -> Option<PCon<T, AnyPolicy>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns)?;
        let table = columns[idx].table_str().into_owned();
        let val = self.row.take(index)?;
        Some(PCon::new(val, get_schema_policies(table, idx, &self.raw)))
    }

    pub fn unwrap(self) -> Vec<PConValue> {
        self.raw
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let columns = self.row.columns_ref();
                let table = columns[i].table_str().into_owned();
                PCon::new(v.clone(), get_schema_policies(table, i, &self.raw))
            })
            .collect()
    }
}
