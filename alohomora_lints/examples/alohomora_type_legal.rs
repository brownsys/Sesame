extern crate alohomora;

use alohomora::{AlohomoraType, AlohomoraTypeEnum};
use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::PrivacyPureRegion;

#[derive(AlohomoraType)]
struct MyStruct {
    x: BBox<i32, NoPolicy>,
}

fn main() {
    let bbox = BBox::new(10i32, NoPolicy {});
    let bbox = bbox.into_ppr(PrivacyPureRegion::new(|x| x + 1));
    let x = bbox.discard_box();
    println!("{}", x);
}