// BBox
use crate::bbox::BBox;
use crate::policy::{AnyPolicyClone, AnyPolicyCloneDyn, AnyPolicyable};

// mysql imports.
pub use mysql::prelude::FromValue as BBoxFromValue;

// What is a (return) value.
pub type BBoxValue = BBox<mysql::Value, AnyPolicyClone>;

// Type modification.
pub fn from_value<T: BBoxFromValue, P: AnyPolicyCloneDyn>(v: BBoxValue) -> Result<BBox<T, P>, String> {
    let (t, p) = v.consume();
    Ok(BBox::new(mysql::from_value(t), p.specialize_top()?))
}
pub fn from_value_or_null<T: BBoxFromValue, P: AnyPolicyCloneDyn>(
    v: BBoxValue,
) -> Result<BBox<Option<T>, P>, String> {
    let (t, p) = v.consume();
    Ok(BBox::new(
        match t {
            mysql::Value::NULL => None,
            t => Some(mysql::from_value(t)),
        },
        p.specialize_top()?,
    ))
}
