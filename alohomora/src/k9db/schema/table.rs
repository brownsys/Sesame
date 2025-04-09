use serde::{Serialize, Deserialize};

use crate::k9db::schema::column::Column;

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    #[serde(default)]
    pub data_subject: bool,
    pub columns: Vec<Column>,
}