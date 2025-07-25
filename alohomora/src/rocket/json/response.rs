use crate::bbox::BBox;
use crate::context::{Context, ContextData, UnprotectedContext};
use crate::policy::{AnyPolicy, Policy, Reason};
use crate::rocket::{
    BBoxRequest, BBoxResponder, BBoxResponse, BBoxResponseResult, ResponseBBoxJson,
};
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde_json::{Map, Value};
use std::collections::HashMap;

// Wrapper around serde_json::Value generated by our types and consumed during responses.
pub enum OutputBBoxValue {
    Value(Value),
    BBox(BBox<Box<OutputBBoxValue>, AnyPolicy>),
    Array(Vec<OutputBBoxValue>),
    Object(HashMap<String, OutputBBoxValue>),
}
impl OutputBBoxValue {
    // Not public, not exposed to applications.
    fn transform(self, context: &UnprotectedContext) -> Result<Value, &'static str> {
        match self {
            OutputBBoxValue::BBox(bbox) => {
                if bbox.policy().check(context, Reason::Response) {
                    bbox.consume().0.transform(context)
                } else {
                    Err("Policy check failed")
                }
            }
            OutputBBoxValue::Value(value) => Ok(value),
            OutputBBoxValue::Array(vec) => {
                let mut v = Vec::with_capacity(vec.len());
                for val in vec {
                    v.push(val.transform(context)?);
                }
                Ok(Value::Array(v))
            }
            OutputBBoxValue::Object(map) => {
                let mut m = Map::with_capacity(map.len());
                for (key, val) in map {
                    m.insert(key, val.transform(context)?);
                }
                Ok(Value::Object(m))
            }
        }
    }
}

// Endpoints can return (T: FromBBoxJson, Context) which Alohomora eventually turns into
// T after a policy check.
pub struct JsonResponse<T: ResponseBBoxJson, D: ContextData>(pub T, pub Context<D>);
impl<T: ResponseBBoxJson, D: ContextData> From<(T, Context<D>)> for JsonResponse<T, D> {
    fn from((json, context): (T, Context<D>)) -> Self {
        Self(json, context)
    }
}

impl<'a, 'r, 'o: 'a, T: ResponseBBoxJson, D: ContextData> BBoxResponder<'a, 'r, 'o>
    for JsonResponse<T, D>
{
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'o> {
        let (json, context) = (self.0, self.1);
        let context = UnprotectedContext::from(context);
        match json.to_json().transform(&context) {
            Err(_) => Err(rocket::http::Status { code: 555 }),
            Ok(json) => {
                let x = Json(json);
                match <Json<Value> as Responder>::respond_to(x, request.get_request()) {
                    Ok(response) => Ok(BBoxResponse::new(response)),
                    Err(e) => Err(e),
                }
            }
        }
    }
}
