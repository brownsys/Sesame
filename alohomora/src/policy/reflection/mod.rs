// This allows performing reflection on Policy structures.
// This is useful to join (rather than stack) during conjunction, and for specialization
// type erased policies to concrete types.

mod r#enum;
mod owned;
mod r#ref;

pub use r#enum::*;
pub use owned::*;
pub use r#ref::*;