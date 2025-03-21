use crate::third::grandchild; 

pub fn child(x: u8) -> u8 {
    println!("stolen secret: {}", x);
    x + grandchild(x)
}