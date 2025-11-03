extern crate alohomora;

use alohomora::sandbox::{AlohomoraSandbox};

pub fn example(a: u32) -> u32 {
  a
}

pub struct Example {}
impl AlohomoraSandbox<u32, u32> for Example {
  /// The actual sandbox function.
  #[cfg(target_arch = "wasm32")]
  fn function(arg: u32) -> u32 { arg }

  /// The FFI function responsible for invoking this sandbox.
  #[cfg(not(target_arch = "wasm32"))]
  fn ffi(arg: *mut std::ffi::c_void, sandbox: usize) -> *mut std::ffi::c_void {
    0 as *mut std::ffi::c_void
  }
}

pub fn main() {}
