
use std::any::Any;
use core::fmt::Debug;
use alohomora::bbox::BBox;
use alohomora::dyns::{SesameDynType, SesameTypeDynTypes};
use alohomora::{SesameTypeDyn, SesameTypeEnumDyn};
use alohomora::policy::{NoPolicy, AnyPolicy};

// Ideally, this becomes a macro.
pub trait AnyDebug: Debug + Any {
    fn upcast_any(&self) -> &dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_debug(&self) -> &dyn Debug;
    fn upcast_debug_box(self: Box<Self>) -> Box<dyn Debug>;

}
impl<T: Debug + Any> AnyDebug for T {
    fn upcast_any(&self) -> &dyn Any { self }
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any> { Box::new(*self) }
    fn upcast_debug(&self) -> &dyn Debug { self }
    fn upcast_debug_box(self: Box<Self>) -> Box<dyn Debug> { Box::new(*self) }
}
// End of Macro.

// Ideally, this is also a derive macro (or maybe directly part of previous macro).
impl SesameDynType for dyn AnyDebug {
    fn upcast_ref(&self) -> &dyn Any {
        self.upcast_any()
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self.upcast_any_box()
    }
}

impl<T: AnyDebug> SesameTypeDynTypes<T> for dyn AnyDebug {
    fn box_me(t: T) -> Box<dyn AnyDebug> {
        Box::new(t)
    }
}
// End of Macro.

pub fn example() {
    let bbox = BBox::new(String::from("hello"), NoPolicy {});
    let mut sesame_enum: SesameTypeEnumDyn<dyn AnyDebug> = bbox.to_enum();
    println!("Whats up");
    if let SesameTypeEnumDyn::BBox(bbox) = sesame_enum {
        let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
        let value = bbox.discard_box();
        println!("{:?}", value);
        sesame_enum = SesameTypeEnumDyn::Value(value);
    }
    let x = BBox::<String, NoPolicy>::from_enum(sesame_enum).unwrap();
    println!("{:?}", x);
}