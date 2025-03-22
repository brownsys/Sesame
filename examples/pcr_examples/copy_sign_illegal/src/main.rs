use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn main() {
    copy_fn_to_dependencies();
    copy_between_fns();
  
}

fn copy_between_fns() {
    // first is correctly signed
    let _pcr = PrivacyCriticalRegion::new(
        |x: u8| { x + 1 },
        Signature {
            username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNQU200aEw3bnRUQi90K3pHcUdtdmpsOVZCS0VVVDdOZFdaNnpaZ21VMExveGlqMTZ4MnMzdkQ5RWJacFZGaWgKRGg5NXEvb3BueW1oUkpUaHBUckJRRAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    ); 

    //second has signature of the first
    let _pcr2 = PrivacyCriticalRegion::new(
        |x: u8| { x + 2 },
        Signature {
            username: "corinnt", // copied signature from _pcr
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNQU200aEw3bnRUQi90K3pHcUdtdmpsOVZCS0VVVDdOZFdaNnpaZ21VMExveGlqMTZ4MnMzdkQ5RWJacFZGaWgKRGg5NXEvb3BueW1oUkpUaHBUckJRRAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    );
}

fn copy_fn_to_dependencies() {
    let _pcr = PrivacyCriticalRegion::new(
        |x: u8| { x + 1 },
        Signature {
            username: "corinnt", 
            signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNQU200aEw3bnRUQi90K3pHcUdtdmpsOVZCS0VVVDdOZFdaNnpaZ21VMExveGlqMTZ4MnMzdkQ5RWJacFZGaWgKRGg5NXEvb3BueW1oUkpUaHBUckJRRAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"
        }
    ); 
}

