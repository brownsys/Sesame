use crate::rocket::{
    FromPConData, PConData, PConDataOutcome, PConJson, PConRequest, RequestPConJson,
};
use rocket::data::ByteUnit;
use serde_json::Value;

// Wrapper around serde_json::Value created from requests and consumed by our types.
pub struct InputPConValue {
    pub(super) value: Value,
}
impl InputPConValue {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
    pub fn get(&mut self, key: &str) -> Result<InputPConValue, &'static str> {
        match self.value.as_object_mut() {
            None => Err("Bad JSON"),
            Some(map) => match map.remove(key) {
                None => Ok(InputPConValue::new(Value::Null)),
                Some(val) => Ok(InputPConValue::new(val)),
            },
        }
    }
    pub fn into_json<T: RequestPConJson>(
        self,
        request: PConRequest<'_, '_>,
    ) -> Result<T, &'static str> {
        T::from_json(self, request)
    }
}

async fn parse_body<'a, 'r>(
    r: PConRequest<'a, 'r>,
    d: PConData<'a>,
) -> Result<String, &'static str> {
    let size_limit = r
        .get_request()
        .limits()
        .get("json")
        .unwrap_or(ByteUnit::Mebibyte(1));
    match d.get_data().open(size_limit).into_string().await {
        Ok(string) => Ok(string.into_inner()),
        Err(_) => Err("Request data is incomplete"),
    }
}

// Allows us to use this as a data parameter in routes.
#[rocket::async_trait]
impl<'a, 'r, T: RequestPConJson> FromPConData<'a, 'r> for PConJson<T> {
    type PConError = &'static str;

    async fn from_data(
        req: PConRequest<'a, 'r>,
        data: PConData<'a>,
    ) -> PConDataOutcome<'a, 'r, Self> {
        match parse_body(req, data).await {
            Err(e) => PConDataOutcome::Failure((rocket::http::Status::BadRequest, e)),
            Ok(json_string) => match serde_json::from_str(&json_string) {
                Ok(v) => match T::from_json(InputPConValue::new(v), req) {
                    Ok(t) => PConDataOutcome::Success(PConJson(t)),
                    Err(e) => PConDataOutcome::Failure((rocket::http::Status::BadRequest, e)),
                },
                Err(_) => PConDataOutcome::Failure((
                    rocket::http::Status::BadRequest,
                    "Request data is not JSON",
                )),
            },
        }
    }
}
