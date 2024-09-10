#![feature(allocator_api)]
extern crate alohomora;
extern crate alohomora_sandbox;

use alohomora::sandbox::{AlohomoraSandbox};

#[AlohomoraSandbox()]
pub fn example(a: u32) -> u32 {
  a
}

pub fn main() {}
