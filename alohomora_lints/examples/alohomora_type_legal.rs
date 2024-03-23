extern crate alohomora;

use alohomora::AlohomoraType;
use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::{execute_pure, PrivacyPureRegion};

// TODO(babman): Derive Macro has some bug: does not generate proper struct here
//               if name is not provided.
#[derive(AlohomoraType)]
#[alohomora_out_type(name = "MyStructLite")]
struct MyStruct {
    x: BBox<i32, NoPolicy>,
}

fn main() {
    let bbox = MyStruct { x: BBox::new(10i32, NoPolicy {}) };
    let bbox = execute_pure(bbox, PrivacyPureRegion::new(|x: MyStructLite| x.x + 1)).unwrap();
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    println!("{}", bbox.discard_box());
}
