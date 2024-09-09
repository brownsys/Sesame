use alohomora_derive::{AlohomoraSandbox, FastTransfer};

use serde::{Serialize, Deserialize};

// Argument type for sandboxes.
#[derive(Serialize, Deserialize)]
pub struct Numbers {
    pub a: u32,
    pub b: u32,
}

#[repr(C)]
#[derive(Serialize, Deserialize, FastTransfer)]
pub struct NumbersFast {
    pub a: u32,
    pub b: u32,
}

// Sandbox functions.
#[AlohomoraSandbox()]
pub fn add_numbers(n: Numbers) -> u32 {
    n.a + n.b
}

#[AlohomoraSandbox()]
pub fn mult_numbers(n: NumbersFast) -> u32 {
  n.a * n.b
}

#[AlohomoraSandbox()]
pub fn div_numbers(n: Numbers) -> u32 {
  // Whether we see these or not depends on whether printing is enabled in
  // `alohomora_build/sandbox_stubs/Makefile`
  println!("my numbers are {} and {} - 1", n.a, n.b);
  println!("my numbers are {} and {} - 2", n.a, n.b);
  println!("my numbers are {} and {} - 3", n.a, n.b);
  println!("my numbers are {} and {} - 4", n.a, n.b);
  n.a / n.b
}
