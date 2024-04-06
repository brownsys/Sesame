use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn main() {
    copy_between_fns();
    copy_fn_to_dependencies();  
}

fn copy_between_fns() {
    let _pcr = PrivacyCriticalRegion::new(|x: u8| { x + 1  },
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""},  
        Signature {username: "corinnt", 
            signature: ""}); 

    let _pcr2 = PrivacyCriticalRegion::new(|x: u8| { x + 2 },
                Signature {username: "corinnt", 
                    signature: ""}, 
                Signature {username: "corinnt", 
                    signature: ""}, 
                Signature {username: "corinnt", 
                    signature: ""});
}

fn copy_fn_to_dependencies() {
    let _pcr = PrivacyCriticalRegion::new(|x: u8| { x + 1  },
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""},  
        Signature {username: "corinnt", 
            signature: ""}); 
}

