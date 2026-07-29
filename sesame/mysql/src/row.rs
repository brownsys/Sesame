use std::sync::Arc;

use sesame::pcon::PCon;
use sesame::policy::AnyPolicy;

use crate::policy::ColumnPolicies;
use crate::{PConFromValue, PConValue};

// mysql imports.
pub use mysql::prelude::ColumnIndex as PConColumnIndex;

// A result row. Carries the per-column policy factories its query
// resolved once, so attaching policies to cells needs no registry
// lookup here.
#[derive(Clone)]
pub struct PConRow {
    row: mysql::Row,
    raw: Vec<mysql::Value>,
    policies: Arc<ColumnPolicies>,
}
impl PConRow {
    pub(super) fn new(row: mysql::Row, policies: Arc<ColumnPolicies>) -> Self {
        let raw = row.clone().unwrap();
        PConRow { row, raw, policies }
    }

    pub fn get<T: PConFromValue, I: PConColumnIndex>(
        &self,
        index: I,
    ) -> Option<PCon<T, AnyPolicy>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns)?;
        let val = self.row.get(index)?;
        Some(PCon::new(val, self.policies.for_cell(idx, &self.raw)))
    }

    pub fn take<T: PConFromValue, I: PConColumnIndex>(
        &mut self,
        index: I,
    ) -> Option<PCon<T, AnyPolicy>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns)?;
        let val = self.row.take(index)?;
        Some(PCon::new(val, self.policies.for_cell(idx, &self.raw)))
    }

    pub fn unwrap(self) -> Vec<PConValue> {
        self.raw
            .iter()
            .enumerate()
            .map(|(i, v)| PCon::new(v.clone(), self.policies.for_cell(i, &self.raw)))
            .collect()
    }
}
