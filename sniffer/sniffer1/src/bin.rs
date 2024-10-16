use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;

pub fn main() {
  let mut vec = Vec::new();
  for i in 0..10 {
    vec.push(BBox::new(i as u32, NoPolicy {}));
  }
  for bbox in vec.iter() {
    unsafe {
      let view = &bbox as *const _ as *const u8;
      let size = std::mem::size_of::<BBox<u32, NoPolicy>>() as isize;
      for i in 0..size {
        print!("{:02x}", *view.offset(i));
      }
      println!("");
    }
  }
}
