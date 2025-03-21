extern crate alohomora; 

use alohomora::pcr::{PrivacyCriticalRegion, Signature};

fn main() {
    let _pcr_all_blank = PrivacyCriticalRegion::new(|x: u8| { println!("pcr_all_blank {}", x) },
        Signature {username: "corinnt", 
            signature: ""}, 
        Signature {username: "corinnt", 
            signature: ""},
        Signature {username: "corinnt", 
            signature: ""}); 

    let _pcr_author_blank = PrivacyCriticalRegion::new(|x: u8| { println!("pcr_author_blank {}", x) },
            Signature {username: "corinnt", 
                signature: ""}, 
            Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNQRjBZbCtzRmRqUWV1V0hCMDhMZSsxQmRlb1B1aTczanZsMXJUY2Q0ZW83Vm9YZWJKMVh2Qi9IQnNqc1Axd1QKNHpEby9ERHhyZklDSG1PWEwwZHRFTgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"},
            Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURzTmdMZ2MrVlZqYmM5ek5jdFV3V0p4NHAvOTVxV2tkdGUvU2FuSzl0NDBxTmEwdjdSQzVXZTcrWU45cVBDNmsKU09RaVZKbG4vZ1hDZWVlamx5MkpBSwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}); 

    let _pcr_fn_rev_blank = PrivacyCriticalRegion::new(|x: u8| { println!("pcr_fn_rev_blank {}", x) },
            Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUE3LzB0Z3BXSjVYL3kzRGp2V3NHaVZUVWJqRmFwSU5KMkV6VTFHQU1oUHpBOCtEZVZmbFc0MUJpcnF1ZzFWc0kKSVZ0cnFXRW9EQU1nT2hVQjVUU3RnQwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}, 
            Signature {username: "corinnt", 
                signature: ""},
            Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURzTmdMZ2MrVlZqYmM5ek5jdFV3V0p4NHAvOTVxV2tkdGUvU2FuSzl0NDBxTmEwdjdSQzVXZTcrWU45cVBDNmsKU09RaVZKbG4vZ1hDZWVlamx5MkpBSwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}); 
    
    let _pcr_lock_rev_blank = PrivacyCriticalRegion::new(|x: u8| { println!("pcr_lock_rev_blank {}", x) },
                Signature {username: "corinnt", 
                    signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNIUnhadEYyZTllT2lobkN3WHU4UmFTRC9pZUpYTGtObkJkYUFQektTRGZFT0VFME1SbEpGK2drcjkwc2RjZXoKU1U1QkhNN21TeDNMRG9hZlRQWVpFSwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}, 
                Signature {username: "corinnt", 
                    signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNIUnhadEYyZTllT2lobkN3WHU4UmFTRC9pZUpYTGtObkJkYUFQektTRGZFT0VFME1SbEpGK2drcjkwc2RjZXoKU1U1QkhNN21TeDNMRG9hZlRQWVpFSwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"},
                Signature {username: "corinnt", 
                    signature: ""}); 
}

