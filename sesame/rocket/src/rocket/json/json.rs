use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

use sesame::extensions::{SesameExtension, UncheckedSesameExtension};
use sesame::pcon::PCon;
use sesame::policy::{AnyPolicy, AnyPolicyable};

use crate::policy::FrontendPolicy;
use crate::rocket::{InputPConValue, OutputPConValue, PConRequest};

// Traits for transformation between JSON data and structs.
pub trait RequestPConJson {
    fn from_json(value: InputPConValue, request: PConRequest<'_, '_>) -> Result<Self, &'static str>
    where
        Self: Sized;
}

pub trait ResponsePConJson {
    fn to_json(self) -> OutputPConValue;
}

// Implement this for predefined types, implementations for custom structs should use the derive
// macro.
// Implement trait for Date types.
impl<T: DeserializeOwned, P: FrontendPolicy> RequestPConJson for PCon<T, P> {
    fn from_json(
        value: InputPConValue,
        request: PConRequest<'_, '_>,
    ) -> Result<Self, &'static str> {
        let value = value.value;
        match serde_json::from_value(value) {
            Err(_) => Err("Bad JSON"),
            Ok(value) => Ok(PCon::new(value, P::from_request(request.get_request()))),
        }
    }
}

// Option (for nulls).
impl<T: RequestPConJson> RequestPConJson for Option<T> {
    fn from_json(value: InputPConValue, request: PConRequest<'_, '_>) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
        if let Value::Null = value.value {
            Ok(None)
        } else {
            Ok(Some(T::from_json(value, request)?))
        }
    }
}

// Request containers.
impl<T: ResponsePConJson> ResponsePConJson for Vec<T> {
    fn to_json(self) -> OutputPConValue {
        OutputPConValue::Array(self.into_iter().map(|v| v.to_json()).collect())
    }
}
impl<T: RequestPConJson> RequestPConJson for HashMap<String, T> {
    fn from_json(value: InputPConValue, request: PConRequest<'_, '_>) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
        match value.value {
            Value::Object(map) => {
                let mut result = HashMap::with_capacity(map.len());
                for (k, v) in map {
                    result.insert(k, T::from_json(InputPConValue::new(v), request)?);
                }
                Ok(result)
            }
            _ => Err("Bad JSON"),
        }
    }
}

// Now, we implement response trait.
// Anything that is json serializable can be made to be a response.
macro_rules! impl_base_types {
    ($T: ty) => {
        impl ResponsePConJson for $T {
            fn to_json(self) -> OutputPConValue {
                OutputPConValue::Value(serde_json::to_value(self).unwrap())
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

// PCon of anything that is ResponsePConJson is also ResponsePConJson.
impl<T: ResponsePConJson, P: AnyPolicyable> ResponsePConJson for PCon<T, P> {
    fn to_json(self) -> OutputPConValue {
        struct Converter {}
        impl UncheckedSesameExtension for Converter {}
        impl<T: ResponsePConJson> SesameExtension<T, AnyPolicy, OutputPConValue> for Converter {
            fn apply(&mut self, data: T, policy: AnyPolicy) -> OutputPConValue {
                OutputPConValue::PCon(PCon::new(Box::new(data.to_json()), policy))
            }
        }
        let pcon = self.into_any_policy_no_clone();
        pcon.unchecked_extension(&mut Converter {})
    }
}

// Response containers.
impl<T: ResponsePConJson> ResponsePConJson for Option<T> {
    fn to_json(self) -> OutputPConValue {
        match self {
            None => OutputPConValue::Value(Value::Null),
            Some(v) => v.to_json(),
        }
    }
}
impl<T: RequestPConJson> RequestPConJson for Vec<T> {
    fn from_json(value: InputPConValue, request: PConRequest<'_, '_>) -> Result<Self, &'static str>
    where
        Self: Sized,
    {
        match value.value {
            Value::Array(vec) => {
                let mut result = Vec::with_capacity(vec.len());
                for v in vec {
                    result.push(T::from_json(InputPConValue::new(v), request)?);
                }
                Ok(result)
            }
            _ => Err("Bad JSON"),
        }
    }
}
impl<T: ResponsePConJson> ResponsePConJson for HashMap<String, T> {
    fn to_json(self) -> OutputPConValue {
        OutputPConValue::Object(self.into_iter().map(|(k, v)| (k, v.to_json())).collect())
    }
}
