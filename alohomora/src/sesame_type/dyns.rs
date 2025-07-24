use std::any::Any;
use erased_serde::Serialize;

// You should implement this for combination of traits you can about preserving through
// SesameType from_enum and into_enum transformation.
// E.g., Tahini should implement this for Serialize + Debug.
pub trait SesameDynType {
    //type DynType;
    fn upcast_ref(&self) -> &dyn Any;
    fn upcast_box(self: Box<Self>) -> Box<dyn Any>;
}

// TODO(babman): this is almost right, but cannot be implemented in foreign crates even for their
// own traits. Need to perhaps flip T and Self! Look at youchat for an example.
// TODO(babman): make sure fold tests pass!
pub trait SesameTypeDynTypes<T> where Self: SesameDynType {
    fn box_me(t: T) -> Box<Self>;
}

// Example: Now we can preserve Serialize + Any through SesameType transformations.
// Traits that we care about preserving inside SesameType.
// This part should be macro-ed:
pub trait AnySerialize: Serialize + Any {
    // These upcasts would be unneeded with trait object upcasting but we are not using a new
    // enough Rust version :(
    fn upcast_any(&self) -> &dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_serialize(&self) -> &dyn Serialize;
    fn upcast_serialize_box(self: Box<Self>) -> Box<dyn Serialize>;
}
impl<T: Serialize + Any> AnySerialize for T {
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
impl SesameDynType for dyn AnySerialize {
    fn upcast_ref(&self) -> &dyn Any {
        self.upcast_any()
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self.upcast_any_box()
    }
}

impl<T: Any + Serialize> SesameTypeDynTypes<T> for dyn AnySerialize {
    fn box_me(t: T) -> Box<dyn AnySerialize> {
        Box::new(t)
    }
}
// End of Macro.

// Provided implementations for types we care about
impl SesameDynType for dyn Any {
    fn upcast_ref(&self) -> &dyn Any {
        self
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
impl<T: Any> SesameTypeDynTypes<T> for dyn Any {
    fn box_me(t: T) -> Box<dyn Any> {
        Box::new(t)
    }
}
// End provided impls.