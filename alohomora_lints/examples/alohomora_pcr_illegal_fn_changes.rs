extern crate alohomora; 

use alohomora::pcr::{PrivacyCriticalRegion, Signature};

//TODO changing the value of a const or static doesn't invalidate the hash. 

static static_num: u8 = 42; 

const u8 CONST_NUM = 7; 

fn main() {
    let static_pcr = PrivacyCriticalRegion::new(|x: u8| { println!(x + static_num) },
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""}); 

    let const_pcr = PrivacyCriticalRegion::new(|x: u8| { println!(x + CONST_NUM) },
    Signature {username: "corinnt", 
        signature: ""}, 
    Signature {username: "corinnt", 
        signature: ""}, 
    Signature {username: "corinnt", 
        signature: ""})
}