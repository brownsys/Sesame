extern crate alohomora;

use alohomora::sandbox::{AlohomoraSandbox};

pub fn example(a: u32) -> u32 {
  a
}

pub struct Example {}
impl<'a, 'b> AlohomoraSandbox<'a, 'b, u32, u32> for Example {
  fn invoke(a: u32) -> u32 {
    println!("{}", a);
    example(a)
  }
}

pub fn main() {}
