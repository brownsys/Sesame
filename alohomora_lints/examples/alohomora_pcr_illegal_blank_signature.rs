extern crate alohomora; 

use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn main() {
    let _pcr = PrivacyCriticalRegion::new(|x: u8| { println!(x) },
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""},
        Signature {username: "corinnt", 
            signature: ""}); 

    let _pcr1 = PrivacyCriticalRegion::new(|x: u8| { println!(x) },
            Signature {username: "corinnt", 
                signature: "kdsfjl;a"}, 
            Signature {username: "corinnt", 
                signature: ""},
            Signature {username: "corinnt", 
                signature: "jljl;j"}); 
    
    let _pcr2 = PrivacyCriticalRegion::new(|x: u8| { println!(x) },
                Signature {username: "corinnt", 
                    signature: "kdsfjl;a"}, 
                Signature {username: "corinnt", 
                    signature: "ljnlj"},
                Signature {username: "corinnt", 
                    signature: ""}); 

    let _pcr3 = PrivacyCriticalRegion::new(|x: u8| { println!(x) },
                    Signature {username: "corinnt", 
                        signature: ""}, 
                    Signature {username: "corinnt", 
                        signature: "ksdfjks"},
                    Signature {username: "corinnt", 
                        signature: "jlj;l"}); 
}

