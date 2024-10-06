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
            username: "KinanBab", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWdRRVVMUGFSOEVlZk53WGtvc2RhZFJDZU14Zwp3MnEvMlY3dzk4VndneUZiTUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNVRW9JaW1QcVVWbDBNbW9VTjJMb2VrdXdSeUpNWDZoekxzSlljK3Fzb1duYk83YWNNUE1zU2RLZ2ljUmp1OWYKTnZLaGF0Rk1kVEFFZGlROWJ0UWRjQQotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    ); 
}
