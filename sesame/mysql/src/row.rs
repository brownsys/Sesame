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
    policies: Arc<ColumnPolicies>,
}
impl PConRow {
    pub(super) fn new(row: mysql::Row, policies: Arc<ColumnPolicies>) -> Self {
        PConRow { row, policies }
    }

    pub fn get<T: PConFromValue, I: PConColumnIndex>(
        &self,
        index: I,
    ) -> Option<PCon<T, AnyPolicy>> {
        let columns = self.row.columns_ref();
        let idx = index.idx(columns)?;
        let val = self.row.get(index)?;
        Some(PCon::new(val, self.policies.for_cell(idx, &self.row)))
    }

    pub fn unwrap(self) -> Vec<PConValue> {
        let mut policies = Vec::with_capacity(self.row.len());
        for i in 0..self.row.len() {
            policies.push(self.policies.for_cell(i, &self.row));
        }
        self.row.unwrap()
            .into_iter()
            .zip(policies)
            .map(|(v, p)| PCon::new(v, p))
            .collect()
    }
}
