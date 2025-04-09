use std::collections::HashMap;
use mysql::Column;
use crate::db::BBoxSetColumns;
use crate::k9db::db::{is_policy_column, parse_policy_column_name};

// Schema
#[derive(Clone)]
pub struct BBoxK9dbColumnSet {
    pub(super) columns: Vec<Column>,
    pub(super) policy_cols: HashMap<String, Vec<(usize, char, String)>>,
}
impl BBoxK9dbColumnSet {
    pub fn new(set: BBoxSetColumns<'_>) -> Self {
        let mut columns = Vec::new();
        let mut policy_cols: HashMap<String, Vec<(usize, char, String)>> = HashMap::new();
        let set_ref = set.as_ref();
        for i in 0..set_ref.len() {
            let col = &set_ref[i];
            let col_name = col.name_str();
            if is_policy_column(col_name.as_ref()) {
                let (op, column_name, policy_name) = parse_policy_column_name(col_name.as_ref());
                // Policy column.
                policy_cols.entry(column_name)
                    .or_default()
                    .push((i, op, policy_name));

            } else {
                // Regular column
                assert_eq!(policy_cols.len(), 0);
                columns.push(col.clone());
            }
        }
        Self { columns, policy_cols }
    }
    pub fn column_index<U: AsRef<str>>(&self, name: U) -> Option<usize> {
        self.columns.iter().position(|col| col.name_str() == name.as_ref())
    }
    pub fn as_ref(&self) -> &[Column] {
        &self.columns
    }
}