use alohomora::pcr::{PrivacyCriticalRegion, Signature};
// toggling the `serde` version (not used in the closure) in the Cargo.toml invalidates the signature
use dependency::external_math; 

fn other_fn(){
    println!("I'm also another function");
}

fn main() {
    let _pcr0 = PrivacyCriticalRegion::new(|x: u8| { call_external_math(x, x) } , 
    Signature {username: "corinnt", 
        signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURUZWw3eTZSQ0lPTnptVXdaWkxSVXAvM2ltaFcvNnhDM2JvSy9uMXpwK2NrV3djNjU1V2RRM09KRHovQ3RDbTcKMVJ2K3lYbU01RjQ4dVUrV2d1NVhJUAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}, 
    Signature {username: "corinnt", 
        signature: ""},
    Signature {username: "corinnt", 
        signature: ""}); 

    let _pcr = PrivacyCriticalRegion::new(|x: u8| { x + x },
        Signature {username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURUZWw3eTZSQ0lPTnptVXdaWkxSVXAvM2ltaFcvNnhDM2JvSy9uMXpwK2NrV3djNjU1V2RRM09KRHovQ3RDbTcKMVJ2K3lYbU01RjQ4dVUrV2d1NVhJUAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}, 
        Signature {username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURUZWw3eTZSQ0lPTnptVXdaWkxSVXAvM2ltaFcvNnhDM2JvSy9uMXpwK2NrV3djNjU1V2RRM09KRHovQ3RDbTcKMVJ2K3lYbU01RjQ4dVUrV2d1NVhJUAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"},
        Signature {username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNEaURlSksxSkdHUlB0N3FzWDYzUkF5VCtJSzRDeFZ1UGEvc1kwMGppSUUwRkY1Vi9GNHRFWGlHS0NIRW5NMVgKNVgwTndqZ01yUk8ybzlCWS9ZK09VTgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}); 
}

fn call_external_math(left: u8, right: u8) -> u8 {
    external_math(left, right)
}
