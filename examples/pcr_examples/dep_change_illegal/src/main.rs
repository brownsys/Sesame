use alohomora::pcr::{PrivacyCriticalRegion, Signature}; 
use dependency::external_math; 

fn main() {
    //toggling the version of `dependency` will break the Cargo.lock fn signatures
    let _pcr = PrivacyCriticalRegion::new(
        |x: u8| { format!("still private: {}", external_math(x, x)) },
        Signature {
            username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURUTmRyUU5JZXdWQW9rN2ZuYVNJc1lydkljZVpqTllaSEdTMWdsbjEzUVJ2ZlVpVFllR1R6RkVlcWZUYjVYdGIKcDAxdGRrRlJnTXhJS1pBTENlNFBzTgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    ); 
}
