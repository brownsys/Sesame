extern crate alohomora;

use alohomora::FastSandboxTransfer;

#[repr(C)]
#[derive(FastSandboxTransfer)]
pub struct Example {
    pub a: u32,
    pub b: u64,
    pub s: String,
}

pub fn main() {}
