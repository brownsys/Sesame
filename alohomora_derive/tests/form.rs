use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy};
use alohomora::rocket::BBoxRequest;
use alohomora_derive::FromBBoxForm;

use std::any::Any;
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
    fn from_request<'a, 'r>(_: &'a BBoxRequest<'a, 'r>) -> Self {
        TmpPolicy {}
    }
    fn from_cookie() -> Self {
        TmpPolicy {}
    }
}

#[derive(FromBBoxForm)]
struct Nested {
    #[allow(dead_code)]
    inner: alohomora::bbox::BBox<String, TmpPolicy>,
}

#[derive(FromBBoxForm)]
struct Simple {
    #[allow(dead_code)]
    f1: alohomora::bbox::BBox<String, TmpPolicy>,
    #[allow(dead_code)]
    f2: Nested,
    #[allow(dead_code)]
    f3: alohomora::bbox::BBox<u8, TmpPolicy>,
}

// TODO(babman): Test Form data is being parsed correctly!
#[test]
fn simple_from_bbox_form_test() {}
