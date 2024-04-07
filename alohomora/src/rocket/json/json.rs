use std::collections::HashMap;
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
impl<P: FrontendPolicy> RequestBBoxJson for BBox<bool, P> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        match value.value {
            Value::Bool(v) => Ok(BBox::new(v, P::from_request(request.get_request()))),
            _ => Err("Bad JSON"),
        }
    }
}
impl<P: Policy + Clone + 'static> ResponseBBoxJson for BBox<bool, P> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::BBox(self.into_any_policy().into_bbox())
    }
}

impl<P: FrontendPolicy> RequestBBoxJson for BBox<String, P> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        match value.value {
            Value::String(v) => Ok(BBox::new(v, P::from_request(request.get_request()))),
            _ => Err("Bad JSON"),
        }
    }
}
impl<P: Policy + Clone + 'static> ResponseBBoxJson for BBox<String, P> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::BBox(self.into_any_policy().into_bbox())
    }
}

// Implement trait for BBox<number types>
impl<P: FrontendPolicy> RequestBBoxJson for BBox<u64, P> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        match value.value {
            Value::Number(v) => match v.as_u64() {
                None => Err("Bad JSON"),
                Some(v) => Ok(BBox::new(v, P::from_request(request.get_request()))),
            },
            _ => Err("Bad JSON"),
        }
    }
}
impl<P: Policy + Clone + 'static> ResponseBBoxJson for BBox<u64, P> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::BBox(self.into_any_policy().into_bbox())
    }
}
impl<P: FrontendPolicy> RequestBBoxJson for BBox<i64, P> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        match value.value {
            Value::Number(v) => match v.as_i64() {
                None => Err("Bad JSON"),
                Some(v) => Ok(BBox::new(v, P::from_request(request.get_request()))),
            },
            _ => Err("Bad JSON"),
        }
    }
}
impl<P: Policy + Clone + 'static> ResponseBBoxJson for BBox<i64, P> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::BBox(self.into_any_policy().into_bbox())
    }
}
impl<P: FrontendPolicy> RequestBBoxJson for BBox<f64, P> {
    fn from_json(value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> where Self: Sized {
        match value.value {
            Value::Number(v) => match v.as_f64() {
                None => Err("Bad JSON"),
                Some(v) => Ok(BBox::new(v, P::from_request(request.get_request()))),
            },
            _ => Err("Bad JSON"),
        }
    }
}
impl<P: Policy + Clone + 'static> ResponseBBoxJson for BBox<f64, P> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::BBox(self.into_any_policy().into_bbox())
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
impl<T: ResponseBBoxJson> ResponseBBoxJson for Option<T> {
    fn to_json(self) -> OutputBBoxValue {
        match self {
            None => OutputBBoxValue::Value(Value::Null),
            Some(v) => v.to_json()
        }
    }
}

// Containers.
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
impl<T: ResponseBBoxJson> ResponseBBoxJson for HashMap<String, T> {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::Object(self.into_iter().map(|(k, v)| (k, v.to_json())).collect())
    }
}

// Additionally, anything that is json serializable can be made to be a response.
macro_rules! impl_base_types {
    ($T: ty) => {
        impl ResponseBBoxJson for $T {
            fn to_json(self) -> OutputBBoxValue {
                OutputBBoxValue::Value(Value::from(self))
            }
        }
    };
}
impl_base_types!(String);
impl_base_types!(bool);
impl_base_types!(u64);
impl_base_types!(i64);
impl_base_types!(f64);
