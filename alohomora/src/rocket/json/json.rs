use std::collections::HashMap;
use mysql::chrono::{NaiveDateTime, NaiveDate, NaiveTime};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

use crate::bbox::BBox;
use crate::policy::{FrontendPolicy, Policy};
use crate::rocket::{BBoxRequest, InputBBoxValue, OutputBBoxValue};

// Traits for transformation between JSON data and structs.
pub trait RequestBBoxJson {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str>
        where
            Self: Sized;
}

pub trait ResponseBBoxJson {
    fn to_json(self) -> OutputBBoxValue;
}

// Implement this for predefined types, implementations for custom structs should use the derive
// macro.
// Implement trait for Date types.
impl<T: DeserializeOwned, P: FrontendPolicy> RequestBBoxJson for BBox<T, P> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> {
        let value = value.value;
        match serde_json::from_value(value) {
            Err(_) => Err("Bad JSON"),
            Ok(value) => Ok(BBox::new(value, P::from_request(request.get_request()))),
        }
    }
}


// Option (for nulls).
impl<T: RequestBBoxJson> RequestBBoxJson for Option<T> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        if let Value::Null = value.value {
            Ok(None)
        } else {
            Ok(Some(T::from_json(value, request)?))
        }
    }
}

// Request containers.
impl<T: ResponseBBoxJson> ResponseBBoxJson for Vec<T> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::Array(self.into_iter().map(|v| v.to_json()).collect())
    }
}
impl<T: RequestBBoxJson> RequestBBoxJson for HashMap<String, T> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        match value.value {
            Value::Object(map) => {
                let mut result = HashMap::with_capacity(map.len());
                for (k, v) in map {
                    result.insert(k, T::from_json(InputBBoxValue::new(v), request)?);
                }
                Ok(result)
            },
            _ => Err("Bad JSON"),
        }
    }
}

// Now, we implement response trait.
// Anything that is json serializable can be made to be a response.
macro_rules! impl_base_types {
    ($T: ty) => {
        impl ResponseBBoxJson for $T {
            fn to_json(self) -> OutputBBoxValue {
                OutputBBoxValue::Value(serde_json::to_value(self).unwrap())
            }
        }
    };
}
impl_base_types!(String);
impl_base_types!(bool);
impl_base_types!(u64);
impl_base_types!(i64);
impl_base_types!(f64);
impl_base_types!(i32);
impl_base_types!(u32);
impl_base_types!(NaiveDateTime);
impl_base_types!(NaiveDate);
impl_base_types!(NaiveTime);

// BBox of anything that is ResponseBBoxJson is also ResponseBBoxJson.
impl<T: ResponseBBoxJson, P: Policy + Clone + 'static> ResponseBBoxJson for BBox<T, P> {
    fn to_json(self) -> OutputBBoxValue {
        let (t, p) = self.into_any_policy().consume();
        OutputBBoxValue::BBox(BBox::new(Box::new(t.to_json()), p))
    }
}

// Response containers.
impl<T: ResponseBBoxJson> ResponseBBoxJson for Option<T> {
    fn to_json(self) -> OutputBBoxValue {
        match self {
            None => OutputBBoxValue::Value(Value::Null),
            Some(v) => v.to_json()
        }
    }
}
impl<T: RequestBBoxJson> RequestBBoxJson for Vec<T> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        match value.value {
            Value::Array(vec) => {
                let mut result = Vec::with_capacity(vec.len());
                for v in vec {
                    result.push(T::from_json(InputBBoxValue::new(v), request)?);
                }
                Ok(result)
            },
            _ => Err("Bad JSON"),
        }
    }
}
impl<T: ResponseBBoxJson> ResponseBBoxJson for HashMap<String, T> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::Object(self.into_iter().map(|(k, v)| (k, v.to_json())).collect())
    }
}
