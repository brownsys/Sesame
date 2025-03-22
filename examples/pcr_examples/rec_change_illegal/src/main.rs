use alohomora::pcr::{PrivacyCriticalRegion, Signature};
use crate::second::child; 

mod second;
mod third; 
// changing the function from mod third (called in second::child) invalidates the signature
fn main() {
    let _pcr = PrivacyCriticalRegion::new(
        |x: u8| { x + child(x) },
        Signature {
            username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUJFbWR5RlVhbyt5RlR0RDh2c3oxb0E5dlNNY1dNT3lzWnBzQjRoUDVsVVlLQW9KRm8xK2JvVDR0QnZHTUdxT2EKZFJmakh4eEwyb2JacGdUQS9HMXFVQQotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    ); 
}
