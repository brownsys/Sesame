extern crate alohomora;
extern crate myapp_lib;

use std::collections::HashSet;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::execute_sandbox;
use alohomora::pure::PrivacyPureRegion;

use myapp_lib::{add_numbers, Numbers};

fn main() {
  let bbox = BBox::new(Numbers { a: 4, b: 15 }, NoPolicy {});
  let bbox = execute_sandbox::<add_numbers, _, _>(bbox);
  let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
  println!("{}", bbox.discard_box());

  // PPR.
  let set = HashSet::from([10u32, 7u32]);
  let bbox = BBox::new(10u32, NoPolicy {});
  let bbox = bbox.into_ppr(PrivacyPureRegion::new(|val| set.contains(&val)));
  println!("{}", bbox.discard_box());
  
  let bbox = BBox::new(5u32, NoPolicy {});
  let bbox = bbox.into_ppr(PrivacyPureRegion::new(|val| {
    println!("Buggy leak {}", val);
    set.contains(&val)
  }));
  println!("{}", bbox.discard_box());
}
