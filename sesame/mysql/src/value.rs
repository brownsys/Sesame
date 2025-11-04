use sesame::extensions::{SesameExtension, UncheckedSesameExtension};
use sesame::pcon::PCon;
use sesame::policy::{AnyPolicy, AnyPolicyable};

// mysql imports.
pub use mysql::prelude::FromValue as PConFromValue;

// What is a (return) value.
pub type PConValue = PCon<mysql::Value, AnyPolicy>;

struct ValueConverter {}
impl UncheckedSesameExtension for ValueConverter {}
impl<T: PConFromValue, P: AnyPolicyable>
    SesameExtension<mysql::Value, AnyPolicy, Result<PCon<T, P>, String>> for ValueConverter
{
    fn apply(&mut self, data: mysql::Value, policy: AnyPolicy) -> Result<PCon<T, P>, String> {
        Ok(PCon::new(mysql::from_value(data), policy.specialize_top()?))
    }
}

// Type modification.
pub fn from_value<T: PConFromValue, P: AnyPolicyable>(v: PConValue) -> Result<PCon<T, P>, String> {
    v.unchecked_extension(&mut ValueConverter {})
}
