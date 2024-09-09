extern crate alohomora;
extern crate myapp_lib;

use std::collections::HashSet;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::execute_sandbox;
use alohomora::pure::PrivacyPureRegion;

use myapp_lib::{Numbers, NumbersFast, mult_numbers, div_numbers};

fn main() {
  // PSR
  let bbox = BBox::new(Numbers { a: 20, b: 4 }, NoPolicy {});
  let bbox = execute_sandbox::<div_numbers, _, _>(bbox);
  // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
  // println!("{:?}", bbox);
  // let bbox = bbox.ret;
  let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
  println!("{}", bbox.discard_box());

  let bbox = BBox::new(NumbersFast { a: 5, b: 10 }, NoPolicy {});
  let bbox = execute_sandbox::<mult_numbers, _, _>(bbox);
  // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
  // println!("{:?}", bbox);
  // let bbox = bbox.ret;
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
