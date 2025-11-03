extern crate alohomora;

use alohomora::sandbox::FastTransfer;

#[repr(C)]
#[derive(FastTransfer)]
pub struct Example {
    pub a: u32,
    pub b: u64,
    pub s: String,
}

pub fn main() {}
