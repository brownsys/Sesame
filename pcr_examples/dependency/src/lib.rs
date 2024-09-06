pub fn some_other_fn() {
    println!("I'm another function"); 
}

pub fn external_math(left: u8, right: u8) -> u8 {
    println!("leaking :("); 
    2 * left + right
}