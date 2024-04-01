extern crate alohomora;
extern crate myapp_lib;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::execute_sandbox;
use myapp_lib::{add_numbers, mult_numbers, Numbers};

pub fn add_numbers_in_sandbox(a: u32, b: u32) -> u32 {
  let bbox = BBox::new(Numbers { a, b }, NoPolicy {});
  let bbox = execute_sandbox::<add_numbers, _, _>(bbox);
  let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
  bbox.discard_box()
}

pub fn multiply_numbers_in_sandbox(a: u32, b: u32) -> u32 {
  let bbox = BBox::new(Numbers { a, b }, NoPolicy {});
  let bbox = execute_sandbox::<mult_numbers, _, _>(bbox);
  let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
  bbox.discard_box()
}

#[test]
fn test_add_numbers_in_sandbox() {
  assert_eq!(add_numbers_in_sandbox(3, 4), 3 + 4);
  assert_eq!(add_numbers_in_sandbox(13, 14), 13 + 14);
  assert_eq!(add_numbers_in_sandbox(6, 5), 5 + 6);
}

#[test]
fn test_multiply_numbers_in_sandbox() {
  assert_eq!(multiply_numbers_in_sandbox(13, 21), 13 * 21);
  assert_eq!(multiply_numbers_in_sandbox(13, 14), 13 * 14);
  assert_eq!(multiply_numbers_in_sandbox(66, 7), 66 * 7);
}

fn main() {
  println!("3 + 4 = {}", add_numbers_in_sandbox(3, 4));
  println!("13 * 21 = {}", add_numbers_in_sandbox(13, 21));
}
