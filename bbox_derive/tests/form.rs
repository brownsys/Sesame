use bbox::rocket::TmpPolicy;
use bbox_derive::FromBBoxForm;

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
