use alohomora::pcr::{PrivacyCriticalRegion, Signature};

//KNOWN BUG changing these values doesn't invalidate the signature
const NUM: u8 = 1; 
static STATIC_NUM: u8 = 3; 

fn main() {
    static_fn(); 
    constant_fn(); 
}

fn static_fn() {
    let _static_pcr = PrivacyCriticalRegion::new(|x: u8| { println!("{}", x + STATIC_NUM) },
        Signature {username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNsU0l3YXVTSGU1MXRocXVDN0JJL0pSdVVHQkFYNXZKNmxaY21uVnlvZ29taTZVR0l0YklDOWgvSUx2YXJrUjUKbU5tN0Q0b3U4L3JSem1SS08yZWRFTgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"});
}

fn constant_fn(){
    let _const_pcr = PrivacyCriticalRegion::new(|x: u8| { println!("{}", x + NUM) },
    Signature {username: "corinnt", 
        signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUQ2R2x6MlVOV21PZGwrM0tQbnU0RGNZYlNiOVZVcmVlQllDU082UkRxb3VyTWtaNFhUOXR3YmVzUUlrRGVIaXIKTm9hd1htL01mcFJTUmNXT1BPZ0JFTQotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"});  
}
