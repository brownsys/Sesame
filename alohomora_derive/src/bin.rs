extern crate alohomora;
use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy};
use alohomora_derive::FromBBoxForm;

use std::any::Any;
use rocket::http::Cookie;
use rocket::Request;

pub struct TmpPolicy {}
impl Policy for TmpPolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &dyn Any) -> bool {
        true
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl FrontendPolicy for TmpPolicy {
    fn from_request(_: &'_ Request<'_>) -> Self {
        TmpPolicy {}
    }
    fn from_cookie<'a, 'r>(_name: &str, _cookie: &'a Cookie<'static>, _request: &'a Request<'r>) -> Self {
        TmpPolicy {}
    }
}

#[derive(FromBBoxForm)]
struct Nested {
    #[allow(dead_code)]
    inner: alohomora::bbox::BBox<String, TmpPolicy>,
}

fn main() {

}