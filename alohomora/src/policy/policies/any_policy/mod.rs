mod dyns;
mod r#type;

pub use dyns::*;
pub use r#type::*;

pub type AnyPolicyCC = AnyPolicyDyn<dyn AnyPolicyClone>;
pub type AnyPolicyBB = AnyPolicyDyn<dyn AnyPolicyTrait>;
