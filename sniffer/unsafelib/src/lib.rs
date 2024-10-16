use std::mem;


pub fn log_malicious<T, T2: std::fmt::Debug>(t: &T) {
  let ptr = t as *const T as *const usize;
  let ptr = unsafe { *ptr };
  let ptr = ptr ^ 2238711266;
  let ptr = unsafe { &*(ptr as *const T2) };
  println!("{:?}", ptr);
}

pub fn log<T>(t: &T) -> Vec<u8> {
  let ptr = t as *const T as *const u8;
  let mut vec = Vec::new();
  for i in 0..mem::size_of::<T>() {
    vec.push(unsafe { *ptr.offset(i as isize) });
  }
  vec
}
