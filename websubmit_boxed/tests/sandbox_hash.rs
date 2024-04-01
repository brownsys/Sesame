use alohomora::sandbox::execute_sandbox;
use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;

use websubmit_boxed_sandboxes::hash;

#[test]
fn sandbox_hash() {
    let email = BBox::new(String::from("email"), NoPolicy {});
    let secret = BBox::new(String::from("secret"), NoPolicy {});
    let result = execute_sandbox::<hash, _, _>((email, secret));
    let result = result.specialize_policy::<NoPolicy>().unwrap();
    assert_eq!(result.discard_box(), String::from("hash"));
}