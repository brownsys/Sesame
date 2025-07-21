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

pub use sesame_type::{r#enum::{SesameTypeEnum, SesameTypeEnumDyn}, r#type::{SesameType, SesameTypeDyn, AnySerialize}};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::AlohomoraType;
