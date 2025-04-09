use std::collections::{HashMap};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor};
use serde::ser::SerializeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyArgs {
    pub name: String,
    #[serde(flatten)]
    pub args: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum Policy {
    None,
    Policy(PolicyArgs),
    And(Vec<PolicyArgs>),
    Or(Vec<PolicyArgs>),
}

impl Default for Policy {
    fn default() -> Policy {
        Policy::None
    }
}

impl Serialize for Policy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            Policy::None => serializer.serialize_none(),
            Policy::Policy(p) => p.serialize(serializer),
            Policy::And(constraints) => {
                let mut serialize_map = serializer.serialize_map(Some(1))?;
                serialize_map.serialize_entry("And", constraints)?;
                serialize_map.end()
            },
            Policy::Or(constraints) => {
                let mut serialize_map = serializer.serialize_map(Some(1))?;
                serialize_map.serialize_entry("Or", constraints)?;
                serialize_map.end()
            }
        }
    }
}
impl<'de> serde::de::Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Null => Ok(Policy::None),
            serde_json::Value::String(str) => {
                assert_eq!(str, "none");
                Ok(Policy::None)
            },
            serde_json::Value::Object(mut map) => {
                if map.contains_key("name") {
                    let object = serde_json::Value::Object(map);
                    Ok(Policy::Policy(serde_json::from_value(object).unwrap()))   
                } else {
                    assert_eq!(map.len(), 1);
                    if map.contains_key("And") {
                        let args = map.remove("And").unwrap();
                        Ok(Policy::And(serde_json::from_value(args).unwrap()))
                    } else {
                        let args = map.remove("Or").unwrap();
                        Ok(Policy::Or(serde_json::from_value(args).unwrap()))
                    }
                }
            },
            _ => {
                panic!("");
            }
        }
    }
}