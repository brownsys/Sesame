extern crate alohomora;

use alohomora::{AlohomoraType, AlohomoraTypeEnum};
use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::{PrivacyPureRegion, execute_pure};

static mut LEAKED: i32 = 0;

struct MyStruct {
    x: BBox<i32, NoPolicy>,
}

// This is a manual implementation of AlohomoraType which leaks the secret.
// This is illegal, we expect it to be rejected by our lints.
impl AlohomoraType for MyStruct {
    type Out = i32;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::BBox(self.x.into_any())
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Value(v) => match v.downcast() {
                Err(_) => Err(()),
                Ok(v) => {
                 unsafe { LEAKED = *v };
                 Ok(*v)
                },
            },
            _ => Err(()),
        }
    }
}


fn main() {
    let bbox = MyStruct { x: BBox::new(10i32, NoPolicy {}) };
    let bbox = execute_pure(bbox, PrivacyPureRegion::new(|x| x + 1)).unwrap();
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    println!("{}", bbox.discard_box());
    println!("Successfully leaked {}", unsafe { LEAKED });
}
