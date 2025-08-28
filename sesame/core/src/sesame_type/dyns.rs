use erased_serde::Serialize;
use std::any::Any;

// You should implement this for combination of traits you can about preserving through
// SesameType from_enum and into_enum transformation.
// E.g., Tahini should implement this for Serialize + Debug.
pub trait SesameDyn: Any {
    fn upcast_ref(&self) -> &dyn Any;
    fn upcast_box(self: Box<Self>) -> Box<dyn Any>;
}

// Relates a type T to the corresponding SesameDyn trait object.
// E.g. related every T: Any to dyn Any.
pub trait SesameDynRelation<T: Any>: SesameDyn {
    fn boxed_dyn(t: T) -> Box<Self>;
}

// Example: Now we can preserve Serialize + Any through SesameType transformations.
// This part should be macro-ed for custom combinations of Any + <traits>.
mod private1 {
    use erased_serde::Serialize;
    use std::any::Any;

    pub trait Sealed {}
    impl<T: Any + Serialize> Sealed for T {}
}
pub trait AnySerialize: Any + Serialize + Send + private1::Sealed {
    // These upcasts would be unneeded with trait object upcasting but we are not using a new
    // enough Rust version :(
    fn upcast_any(&self) -> &dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_serialize(&self) -> &dyn Serialize;
    fn upcast_serialize_box(self: Box<Self>) -> Box<dyn Serialize>;
}
impl<T: Any + Serialize + Send> AnySerialize for T {
    fn upcast_any(&self) -> &dyn Any {
        self
    }
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any> {
        Box::new(*self)
    }
    fn upcast_serialize(&self) -> &dyn Serialize {
        self
    }
    fn upcast_serialize_box(self: Box<Self>) -> Box<dyn Serialize> {
        Box::new(*self)
    }
}
impl SesameDyn for dyn AnySerialize {
    fn upcast_ref(&self) -> &dyn Any {
        self.upcast_any()
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self.upcast_any_box()
    }
}

impl<T: Any + Serialize + Send> SesameDynRelation<T> for dyn AnySerialize {
    fn boxed_dyn(t: T) -> Box<dyn AnySerialize> {
        Box::new(t)
    }
}
// End of Macro.

// Provided implementations for types we care about
impl SesameDyn for dyn Any {
    fn upcast_ref(&self) -> &dyn Any {
        self
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
impl<T: Any> SesameDynRelation<T> for dyn Any {
    fn boxed_dyn(t: T) -> Box<dyn Any> {
        Box::new(t)
    }
}
// End provided impls.
