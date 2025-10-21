#![feature(specialization)]
#![feature(negative_impls)]
#![feature(auto_traits)]

#[cfg(test)]
#[macro_use]
extern crate static_assertions;

// Re-export our derive macros
#[cfg(feature = "alohomora_derive")]
extern crate alohomora_derive;
extern crate alohomora_sandbox;

// Export these
pub mod bbox;
pub mod context;
pub mod error;
pub mod extensions;
pub mod fold;
pub mod fold_in;
pub mod pcr;
pub mod policy;
pub mod pure;
pub mod sandbox;
pub mod testing;
pub mod unbox;

// Export this directly under sesame::
mod sesame_type;

pub use sesame_type::{
    dyns as sesame_type_dyns, r#enum::SesameTypeEnum, r#type::SesameType, r#type::SesameTypeOut,
};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::SesameType;
