use rocket::data::ByteUnit;
use serde_json::Value;
use crate::rocket::{BBoxData, BBoxDataOutcome, BBoxJson, BBoxRequest, FromBBoxData, RequestBBoxJson};

// Wrapper around serde_json::Value created from requests and consumed by our types.
pub struct InputBBoxValue {
    pub(super) value: Value,
}
impl InputBBoxValue {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
    pub fn get(&mut self, key: &str) -> Result<InputBBoxValue, &'static str> {
        match self.value.as_object_mut() {
            None => Err("Bad JSON"),
            Some(map) => match map.remove(key) {
                None => Ok(InputBBoxValue::new(Value::Null)),
                Some(val) => Ok(InputBBoxValue::new(val)),
            }
        }
    }
    pub fn into_json<T: RequestBBoxJson>(self, request: BBoxRequest<'_, '_>) -> Result<T, &'static str> {
        T::from_json(self, request)
    }
}

async fn parse_body<'a, 'r>(r: BBoxRequest<'a, 'r>, d: BBoxData<'a>) -> Result<String, &'static str> {
    let size_limit = r.get_request().limits().get("json").unwrap_or(ByteUnit::Mebibyte(1));
    match d.get_data().open(size_limit).into_string().await {
        Ok(string) => Ok(string.into_inner()),
        Err(_) => Err("Request data is incomplete"),
    }
}

// Allows us to use this as a data parameter in routes.
#[rocket::async_trait]
impl<'a, 'r, T: RequestBBoxJson> FromBBoxData<'a, 'r> for BBoxJson<T> {
    type BBoxError = &'static str;

    async fn from_data(req: BBoxRequest<'a, 'r>, data: BBoxData<'a>) -> BBoxDataOutcome<'a, 'r, Self> {
        match parse_body(req, data).await {
            Err(e) => BBoxDataOutcome::Failure((rocket::http::Status::BadRequest, e)),
            Ok(json_string) => match serde_json::from_str(&json_string) {
                Ok(v) => match T::from_json(InputBBoxValue::new(v), req) {
                    Ok(t) => BBoxDataOutcome::Success(BBoxJson(t)),
                    Err(e) => BBoxDataOutcome::Failure((rocket::http::Status::BadRequest, e))
                },
                Err(_) => BBoxDataOutcome::Failure((rocket::http::Status::BadRequest, "Request data is not JSON")),
            }
        }
    }
}