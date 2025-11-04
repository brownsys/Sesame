extern crate example_sandbox_lib;
extern crate sesame;

use sesame::pcon::PCon;
use sesame::policy::{AnyPolicyDyn, NoPolicy};
use sesame::sandbox::execute_sandbox;

use example_sandbox_lib::{add_numbers, div_numbers, mult_numbers, Numbers, NumbersFast};

#[test]
fn sandbox_test() {
    // PSR
    let pcon = PCon::new(Numbers { a: 111, b: 33 }, NoPolicy {});
    let pcon = execute_sandbox::<add_numbers, _, _, dyn AnyPolicyDyn>(pcon);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", pcon);
    // let pcon = pcon.ret;
    let pcon = pcon.specialize_policy::<NoPolicy>().unwrap();
    assert_eq!(pcon.discard_box(), 111 + 33);

    let pcon = PCon::new(Numbers { a: 20, b: 4 }, NoPolicy {});
    let pcon = execute_sandbox::<div_numbers, _, _, dyn AnyPolicyDyn>(pcon);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", pcon);
    // let pcon = pcon.ret;
    let pcon = pcon.specialize_policy::<NoPolicy>().unwrap();
    assert_eq!(pcon.discard_box(), 20 / 4);

    let pcon = PCon::new(NumbersFast { a: 5, b: 10 }, NoPolicy {});
    let pcon = execute_sandbox::<mult_numbers, _, _, dyn AnyPolicyDyn>(pcon);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", pcon);
    // let pcon = pcon.ret;
    let pcon = pcon.specialize_policy::<NoPolicy>().unwrap();
    assert_eq!(pcon.discard_box(), 5 * 10);
}

fn main() {
    // PSR
    let pcon = PCon::new(Numbers { a: 111, b: 33 }, NoPolicy {});
    let pcon = execute_sandbox::<add_numbers, _, _, dyn AnyPolicyDyn>(pcon);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", pcon);
    // let pcon = pcon.ret;
    let pcon = pcon.specialize_policy::<NoPolicy>().unwrap();
    println!("{} = {}", pcon.discard_box(), 111 + 33);

    let pcon = PCon::new(Numbers { a: 20, b: 4 }, NoPolicy {});
    let pcon = execute_sandbox::<div_numbers, _, _, dyn AnyPolicyDyn>(pcon);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", pcon);
    // let pcon = pcon.ret;
    let pcon = pcon.specialize_policy::<NoPolicy>().unwrap();
    println!("{} = {}", pcon.discard_box(), 20 / 4);

    let pcon = PCon::new(NumbersFast { a: 5, b: 10 }, NoPolicy {});
    let pcon = execute_sandbox::<mult_numbers, _, _, dyn AnyPolicyDyn>(pcon);
    // To record and print timing info, set "sandbox_timing" feature in Cargo.toml in myapp and myapp_lib
    // println!("{:?}", pcon);
    // let pcon = pcon.ret;
    let pcon = pcon.specialize_policy::<NoPolicy>().unwrap();
    println!("{} = {}", pcon.discard_box(), 5 * 10);
}
