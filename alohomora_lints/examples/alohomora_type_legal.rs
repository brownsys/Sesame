extern crate alohomora;

use alohomora::AlohomoraType;
use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::{execute_pure, PrivacyPureRegion};

#[derive(AlohomoraType)]
struct MyStruct {
    x: BBox<i32, NoPolicy>,
}

fn main() {
    let bbox = MyStruct { x: BBox::new(10i32, NoPolicy {}) };
    let bbox = execute_pure(bbox, PrivacyPureRegion::new(|x: MyStructOut| x.x + 1)).unwrap();
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    println!("{}", bbox.discard_box());
}
