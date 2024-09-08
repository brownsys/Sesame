use alohomora_derive::AlohomoraSandbox;

use serde::{Serialize, Deserialize};

// Argument type for sandboxes.
#[derive(Serialize, Deserialize)]
pub struct Numbers {
    pub a: u32,
    pub b: u32,
}

// Sandbox functions.
#[AlohomoraSandbox()]
pub fn add_numbers(n: Numbers) -> u32 {
    n.a + n.b
}

#[AlohomoraSandbox()]
pub fn mult_numbers(n: Numbers) -> u32 {
  n.a * n.b
}

#[AlohomoraSandbox()]
pub fn div_numbers(n: Numbers) -> u32 {
  println!("my numbers are {} and {} - 1", n.a, n.b);
  println!("my numbers are {} and {} - 2", n.a, n.b);
  println!("my numbers are {} and {} - 3", n.a, n.b);
  n.a / n.b
}
