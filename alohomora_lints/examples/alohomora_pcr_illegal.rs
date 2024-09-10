use alohomora::pcr::{PrivacyCriticalRegion, Signature};

// changing the function from mod third (called in second::child) invalidates the signature
fn main() {
    let _pcr = PrivacyCriticalRegion::new(
        |x: u8| {
            x + child(x)
        },
        Signature {
            username: "KinanBab", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdRRVVMUGFSOEVlZk53WGtvc2RhZFJDZU14Zwp3MnEvMlY3dzk4VndneUZiTUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUFTNGs4SU9tV1dGb3Avdk5Hb2NtSmNvQWdzOG82OUFQUFBUd3ZlUGVGQ3Z4dTN1amNaaFlpZThDSTZ3aGJFNHAKY1AxclAvVDNxN0l3dy9VY3MyZ1JFTAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }, 
        Signature {
            username: "KinanBab", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdRRVVMUGFSOEVlZk53WGtvc2RhZFJDZU14Zwp3MnEvMlY3dzk4VndneUZiTUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUFTNGs4SU9tV1dGb3Avdk5Hb2NtSmNvQWdzOG82OUFQUFBUd3ZlUGVGQ3Z4dTN1amNaaFlpZThDSTZ3aGJFNHAKY1AxclAvVDNxN0l3dy9VY3MyZ1JFTAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        },
        Signature {
            username: "KinanBab", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdRRVVMUGFSOEVlZk53WGtvc2RhZFJDZU14Zwp3MnEvMlY3dzk4VndneUZiTUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUFsR3VYcnF3TG90UEZmd3FwRVpDK1ZHTEgzSmdtTTgzSGlVK0Y0WXBkYzFyWmp5V1JNT1FGeGVMbUNwRDdrZTUKb2RZZzlTYytkK050ZFNEL2hpeGNVTwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    );
}

pub fn child(x: u8) -> u8 {
    println!("stolen secret: {}", x);
    x + grandchild(x)
}

pub fn grandchild(x: u8) -> u8 {
    // I've changed the below line since signing, this will invalidate the signature!
    println!("leaking secrets: {}", x);
    x
}
