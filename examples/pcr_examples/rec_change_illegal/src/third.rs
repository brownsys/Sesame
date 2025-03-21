pub fn grandchild(x: u8) -> u8 {
    println!("leaking secrets{}", x);  // un-commenting this invalidates the signature in main.rs
    x
}