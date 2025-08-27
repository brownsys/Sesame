#![feature(box_into_inner)]
#![feature(specialization)]
#![feature(core_intrinsics)]
#![feature(negative_impls)]
#![feature(auto_traits)]

#[macro_use]
extern crate lazy_static;

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
pub mod db;
pub mod fold;
pub mod fold_in;
#[cfg(feature = "orm")]
pub mod orm;
pub mod pcr;
pub mod policy;
pub mod pure;
pub mod rocket;
pub mod sandbox;
pub mod testing;
pub mod unbox;

// Export this directly under sesame::
mod sesame_type;
mod extensions;

pub use sesame_type::{
    dyns as sesame_type_dyns, r#enum::SesameTypeEnum, r#type::SesameType, r#type::SesameTypeOut,
};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::SesameType;
