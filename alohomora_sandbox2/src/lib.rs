#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(vec_into_raw_parts)]
#![feature(specialization)]

pub extern crate bincode;
pub extern crate serde;

mod instance;

mod fast_transfer;
mod sandbox;
mod sandboxable;
#[cfg(not(target_arch = "wasm32"))]
mod pointers;

// Expose only public facing API.
pub use instance::SandboxInstance;
pub use sandbox::AlohomoraSandbox;
pub use sandboxable::SandboxableType;
pub use fast_transfer::FastSandboxTransfer;