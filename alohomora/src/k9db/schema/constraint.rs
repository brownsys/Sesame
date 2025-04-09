use std::fmt::Formatter;

use serde::{Serialize, Deserialize, Deserializer, Serializer};
use serde::de::{Error, Visitor};

#[derive(Debug, Clone)]
pub enum Constraint {
    Bool(bool),
    String(String),
}
impl Constraint {
    pub fn target_column(&self) -> String {
        match self {
            Constraint::Bool(b) => panic!(""),
            Constraint::String(s) => {
                let mut result = s.replace('.', "(");
                result.push(')');
                result
            },
        }
    }
}

// Implement serialize and deserialize for constraint value.
struct ConstraintVisitor;

impl<'de> Visitor<'de> for  ConstraintVisitor {
    type Value = Constraint;
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a bool or a string for foreign key targets")
    }
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> where E: Error {
        Ok(Constraint::Bool(v))
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        Ok(Constraint::String(v.to_owned()))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
        Ok(Constraint::String(v))
    }
}

impl Serialize for Constraint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer
    {
        match self {
            Constraint::Bool(v) => serializer.serialize_bool(*v),
            Constraint::String(v) => serializer.serialize_str(v),
        }
    }
}
impl<'de> serde::de::Deserialize<'de> for Constraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(ConstraintVisitor)
    }
}