use serde::{Deserialize, Serialize};
use alohomora_derive::AlohomoraSandbox;

// Sandbox functions.
#[AlohomoraSandbox()]
pub fn hash(inputs: (String, String)) -> String {
    String::from("hash")
}