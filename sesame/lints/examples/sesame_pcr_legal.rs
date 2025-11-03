use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn some_math(left: u8, right: u8) -> u8 {
    // this comment also doesn't matter
   left + right + 42
}

fn main() {
    let _pcr0 = PrivacyCriticalRegion::new(
        |x: u8| {
            //adding this comment doesn't change the hash
           some_math(x, x)
        }, 
        Signature {
            username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURZQ21SWkJtOEQ2MkhTamlDVXNVeVJZY1VITzQ3WVZJSVVYY3g1Q2d3ODlkMm5qY0tyMFFKNTBpSlR2VjRHcmsKbzYySUZoR1pzRHB4T1RkRU1Hd0k4SQotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    ); 
}
