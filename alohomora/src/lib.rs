#![feature(box_into_inner)]
#![feature(specialization)]
#![feature(core_intrinsics)]
#![feature(type_changing_struct_update)]

#[macro_use]
extern crate lazy_static;

// Re-export our derive macros
#[cfg(feature = "alohomora_derive")]
extern crate alohomora_derive;
extern crate alohomora_sandbox;

// Export these
pub mod bbox;
pub mod context;
pub mod db;

#[cfg(feature = "orm")]
pub mod orm;

pub mod policy;
pub mod rocket;
pub mod sandbox;
pub mod testing;
pub mod fold;
pub mod pcr;
pub mod pure;
pub mod unbox;

#[cfg(feature = "tahini")]
pub mod tarpc;

// Export this directly under alohomora::
mod r#type;
pub use r#type::*;

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::AlohomoraType;

#[cfg(feature = "tahini")]
pub use alohomora_derive::{TahiniType, tahini_service};
