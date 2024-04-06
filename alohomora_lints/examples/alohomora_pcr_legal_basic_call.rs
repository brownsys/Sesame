extern crate alohomora; 

use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn child(x: String){
    println!(x)
}
fn main() {
    let _pcr = PrivacyCriticalRegion::new(|x: String| { child(x) },
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""}); 
}