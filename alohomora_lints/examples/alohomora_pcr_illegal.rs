use alohomora::pcr::{PrivacyCriticalRegion, Signature};

// changing the function from mod third (called in second::child) invalidates the signature
fn main() {
    let _pcr = PrivacyCriticalRegion::new(
        |x: u8| {
            x + child(x)
        },
        Signature {
            username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUFDeWgwVGw0T0NkVm53MjJmQlRVcCtPSmtFNk5qWDdKMUVWUzh4SVlzL0JORkhxZHRCSk85OURyKy9IcXdaSFAKVldlc1A1bTQ5TzNrTEprMlFrNUhVQgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        },
     
    );
}

pub fn child(x: u8) -> u8 {
    x + grandchild(x)
}

pub fn grandchild(x: u8) -> u8 {
    // I've added the below line since signing, this will invalidate the signature!
    println!("leaking secrets: {}", x);
    x
}
