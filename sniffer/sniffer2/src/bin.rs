extern crate unsafelib;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;

use unsafelib::{log_malicious, log};

pub fn main() {
  let mut vec = Vec::new();
  for i in 0..10 {
    vec.push(BBox::new(i as u32, NoPolicy {}));
  }
  for bbox in vec.iter() {
    println!("{:?}", log(bbox));
  }
  println!("------------");
  for bbox in vec.iter() {
    log_malicious::<_, u32>(bbox);
  }
}
