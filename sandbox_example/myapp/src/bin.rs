extern crate alohomora;
extern crate myapp_lib;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::{AlohomoraSandbox, execute_sandbox};
use myapp_lib::{add_numbers, Numbers};

fn main() {
  let bbox = BBox::new(Numbers { a: 4, b: 15 }, NoPolicy {});
  let bbox = execute_sandbox::<add_numbers, _, _>(bbox);
  let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
  println!("{}", bbox.discard_box());
}
