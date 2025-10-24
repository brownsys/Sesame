extern crate example_sandbox_lib;
extern crate sesame;

use sesame::bbox::BBox;
use sesame::policy::{AnyPolicyDyn, NoPolicy};
use sesame::sandbox::execute_sandbox;

use example_sandbox_lib::{add_numbers, div_numbers, mult_numbers, Numbers, NumbersFast};

#[test]
fn sandbox_test() {
    // PSR
    let bbox = BBox::new(Numbers { a: 111, b: 33 }, NoPolicy {});
    let bbox = execute_sandbox::<add_numbers, _, _, dyn AnyPolicyDyn>(bbox);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", bbox);
    // let bbox = bbox.ret;
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    assert_eq!(bbox.discard_box(), 111 + 33);

    let bbox = BBox::new(Numbers { a: 20, b: 4 }, NoPolicy {});
    let bbox = execute_sandbox::<div_numbers, _, _, dyn AnyPolicyDyn>(bbox);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", bbox);
    // let bbox = bbox.ret;
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    assert_eq!(bbox.discard_box(), 20 / 4);

    let bbox = BBox::new(NumbersFast { a: 5, b: 10 }, NoPolicy {});
    let bbox = execute_sandbox::<mult_numbers, _, _, dyn AnyPolicyDyn>(bbox);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", bbox);
    // let bbox = bbox.ret;
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    assert_eq!(bbox.discard_box(), 5 * 10);
}

fn main() {
    // PSR
    let bbox = BBox::new(Numbers { a: 111, b: 33 }, NoPolicy {});
    let bbox = execute_sandbox::<add_numbers, _, _, dyn AnyPolicyDyn>(bbox);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", bbox);
    // let bbox = bbox.ret;
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    println!("{} = {}", bbox.discard_box(), 111 + 33);

    let bbox = BBox::new(Numbers { a: 20, b: 4 }, NoPolicy {});
    let bbox = execute_sandbox::<div_numbers, _, _, dyn AnyPolicyDyn>(bbox);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", bbox);
    // let bbox = bbox.ret;
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    println!("{} = {}", bbox.discard_box(), 20 / 4);

    let bbox = BBox::new(NumbersFast { a: 5, b: 10 }, NoPolicy {});
    let bbox = execute_sandbox::<mult_numbers, _, _, dyn AnyPolicyDyn>(bbox);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", bbox);
    // let bbox = bbox.ret;
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    println!("{} = {}", bbox.discard_box(), 5 * 10);
}
