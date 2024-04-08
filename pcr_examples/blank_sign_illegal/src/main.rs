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
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNVUnpvamJNUjNUM0FPaWpqaXpmT2xFdmdrUnpsT28vaCttWTVmRkZrMmVJZmYrdEJ5SVdkQU51RzlBODB6QloKZkhqNnlZRERrQ0hTOTM3dE9IdVhrRwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"},
            Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURxRW9Wam1CUFRCaWtBdzlHS2JqUi9TL2s2bDJYUXZtT2JMdzNEU1pBak9HcTBIL3BBTmJCYlBJb0FaaGZMSnIKT3RmdEpBa255dWRwWjR4ZWZlR2Q0RgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}); 

    let _pcr_fn_rev_blank = PrivacyCriticalRegion::new(|x: u8| { println!("pcr_fn_rev_blank {}", x) },
            Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNVNEswSy9KTk42N0p4OCtoMWljbXZCYWVrNVZ2SWMzUlVEeVFJSlA0amtjY0g2RkpLdEJ6NUQ0c29rcGIyZ3EKVUdtK1c0MzBGNmV0STU1cS9VQmxVSAotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}, 
            Signature {username: "corinnt", 
                signature: ""},
            Signature {username: "corinnt", 
                signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRURxRW9Wam1CUFRCaWtBdzlHS2JqUi9TL2s2bDJYUXZtT2JMdzNEU1pBak9HcTBIL3BBTmJCYlBJb0FaaGZMSnIKT3RmdEpBa255dWRwWjR4ZWZlR2Q0RgotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}); 
    
    let _pcr_lock_rev_blank = PrivacyCriticalRegion::new(|x: u8| { println!("pcr_lock_rev_blank {}", x) },
                Signature {username: "corinnt", 
                    signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUN0aHgzc28ydHNqaUJsQ09IalU2d3ZaWVJNYWJGQWsrZ2lBWWNYWTVKVU1oNXFINEVlK3RCRGxnTFVWdVVZMjYKNENwY3FyQzdpSGQvNXoraGVmL2FFTQotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}, 
                Signature {username: "corinnt", 
                    signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUN0aHgzc28ydHNqaUJsQ09IalU2d3ZaWVJNYWJGQWsrZ2lBWWNYWTVKVU1oNXFINEVlK3RCRGxnTFVWdVVZMjYKNENwY3FyQzdpSGQvNXoraGVmL2FFTQotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"},
                Signature {username: "corinnt", 
                    signature: ""}); 
}

