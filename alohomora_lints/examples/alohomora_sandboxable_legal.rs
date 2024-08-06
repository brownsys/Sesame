extern crate alohomora;

use alohomora::Sandboxable;

#[repr(C)]
#[derive(Sandboxable)]
pub struct Example {
    pub a: u32,
    pub b: u64,
    pub s: String,
}

pub fn main() {}
