pub fn grandchild(x: u8) -> u8 {
    println!("leaking secrets{}", x);  //this change invalidates the signature in main.rs
    x
}