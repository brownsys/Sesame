#![feature(specialization)]
#![feature(negative_impls)]
#![feature(auto_traits)]

#[cfg(test)]
#[macro_use]
extern crate static_assertions;

// Re-export our derive macros
#[cfg(feature = "derive")]
extern crate sesame_derive;
extern crate sesame_sandbox;

// Export these
pub mod context;
pub mod error;
pub mod extensions;
pub mod fold;
pub mod fold_in;
pub mod pcon;
pub mod critical;
pub mod policy;
pub mod verified;
pub mod sandbox;
pub mod testing;

// Export this directly under sesame::
mod sesame_type;

pub use sesame_type::{
    dyns as sesame_type_dyns, r#enum::SesameTypeEnum, r#type::SesameType, r#type::SesameTypeOut,
};

#[cfg(feature = "derive")]
pub use sesame_derive::SesameType;
