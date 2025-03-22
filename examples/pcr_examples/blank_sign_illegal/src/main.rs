extern crate alohomora; 

use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn main() {
    let _pcr_all_blank = PrivacyCriticalRegion::new(
      |x: u8| { println!("pcr_all_blank {}", x) },
      Signature {username: "corinnt", signature: ""}
    );
}

