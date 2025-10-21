// BBox
use sesame::bbox::BBox;
use sesame::extensions::{SesameExtension, UncheckedSesameExtension};
use sesame::policy::{AnyPolicy, AnyPolicyable};

// mysql imports.
pub use mysql::prelude::FromValue as BBoxFromValue;

// What is a (return) value.
pub type BBoxValue = BBox<mysql::Value, AnyPolicy>;

struct ValueConverter {}
impl UncheckedSesameExtension for ValueConverter {}
impl<T: BBoxFromValue, P: AnyPolicyable>
    SesameExtension<mysql::Value, AnyPolicy, Result<BBox<T, P>, String>> for ValueConverter
{
    fn apply(&mut self, data: mysql::Value, policy: AnyPolicy) -> Result<BBox<T, P>, String> {
        Ok(BBox::new(mysql::from_value(data), policy.specialize_top()?))
    }
}

// Type modification.
pub fn from_value<T: BBoxFromValue, P: AnyPolicyable>(v: BBoxValue) -> Result<BBox<T, P>, String> {
    v.unchecked_extension(&mut ValueConverter {})
}
