#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(vec_into_raw_parts)]
#![feature(specialization)]

pub extern crate bincode;
pub extern crate serde;

mod fast_transfer;
mod instance;
#[cfg(not(target_arch = "wasm32"))]
mod pointers;
mod sandbox;
mod sandboxable;
#[cfg(feature = "sandbox_timing")]
mod timings;

// Expose only public facing API.
pub use fast_transfer::{FastTransfer, IdentityFastTransfer};
pub use instance::SandboxInstance;
pub use sandbox::SesameSandbox;
pub use sandboxable::SandboxableType;

#[cfg(feature = "sandbox_timing")]
pub type SandboxOut<R> = timings::SandboxTimingInfo<R>;

#[cfg(not(feature = "sandbox_timing"))]
pub type SandboxOut<R> = R;
