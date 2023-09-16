use bbox::policy::{FrontendPolicy, Policy};
use bbox::rocket::BBoxRequest;
use bbox_derive::FromBBoxForm;
use std::any::Any;
pub struct TmpPolicy {}
impl Policy for TmpPolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &dyn Any) -> bool {
        true
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
    inner: bbox::bbox::BBox<String, TmpPolicy>,
}

#[derive(FromBBoxForm)]
struct Simple {
    f1: bbox::bbox::BBox<String, TmpPolicy>,
    f2: Nested,
    f3: bbox::bbox::BBox<u8, TmpPolicy>,
}

#[test]
fn simple_from_bbox_form_test() {}
