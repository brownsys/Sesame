use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::k9db::schema::constraint::Constraint;
use crate::k9db::schema::policy::Policy;

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub ty: String,
    #[serde(default)]
    #[serde(flatten)]
    pub constraints: HashMap<String, Constraint>,
    #[serde(default)]
    pub policy: Policy,
}