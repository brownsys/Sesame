// use alohomora_derive::AlohomoraSandbox;
// use age::Recipient;
// use base64::Engine;
// use base64::engine::general_purpose::STANDARD as base64;
// use alohomora_sandbox::alloc_mem_in_sandbox;
#![feature(allocator_api)]
use chrono::naive::NaiveDateTime;
use chrono::Utc;
use linfa::dataset::Dataset;
use linfa::prelude::*;
use linfa_linear::{FittedLinearRegression, LinearRegression};
use ndarray::prelude::*;
use sha2::{Digest, Sha256};
use alohomora_derive::AlohomoraSandbox;
use std::convert::TryInto;
use std::os::raw::c_void;
use std::ptr::NonNull;
// use std::str::FromStr;
use std::time::{Duration, Instant};

use std::io::{Read, Write};
use std::{iter, ptr};

extern crate once_cell;

// static mut GLOBAL: u64 = 0;

// #[AlohomoraSandbox()]
// pub fn global_test(input: String) -> u64 {
//   unsafe {
//     let x: u64 = GLOBAL;
//     GLOBAL += 1;
//     x
// }
// }

// Sandbox functions.
// #[AlohomoraSandbox()]
// pub fn hash(inputs: (String, String, u64)) -> (u64, String, u64) {
//   // END TIMER (start in bin)
//   // let start = Utc::now().timestamp_nanos_opt().unwrap() as u64;
//   let now = Instant::now();
//   // let setup = now - inputs.2;

//   let mut hasher = Sha256::new();
//   hasher.update(&inputs.0);
//   hasher.update(&inputs.1);
//   let key = format!("{:x}", hasher.clone().finalize());

//   // println!("im in the sandbox");
//   // println!("your hash is {:x}", hasher.finalize());

//   // START TIMER (end in bin)
//   // let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;
//   (now.elapsed().as_nanos() as u64, key, 0)
// }

#[derive(Debug)]
pub struct TestStruct {
    my_int: u32,
    my_float: f32,
    my_float2: f64, 
    ptr_to_buddy: *mut i32,
}

#[AlohomoraSandbox()]
pub fn train2(inputs: Vec<(NaiveDateTime, u64)>) -> (u64, (), u64) {
  // let vec_ptr: *mut Grandparent = inputs as *mut Grandparent;
  

  unsafe {
    // let mut b = Box::from_raw(inputs);
    let b = &inputs;

    println!("in the sandbox, the struct is {:?}", *b);
    println!("w len {:?} and cap {:?}", (*b).len(), (*b).capacity());
    
    // println!("favorite kid is {:?}", *(*b).favorite_kid);
    // println!("their favorite kid is {:?}", *(*(*b).favorite_kid).favorite_kid);
  }
  (0, (), 1)
}

#[AlohomoraSandbox()]
// pub fn train(inputs: (Vec<(NaiveDateTime, u64)>, u64)) -> (u64, FittedLinearRegression<f64>, u64) {
pub fn train(inputs: Vec<(NaiveDateTime, u64)>) -> (u64, FittedLinearRegression<f64>, u64) {
  // END TIMER (start in bin)
  // let start = Utc::now().timestamp_nanos_opt().unwrap() as u64;
  // println!("in the sandbox, inputs are {:?}", inputs);
  let now = Instant::now();
  // println!("training on vec with size {} and cap {}", inputs.len(), inputs.capacity());
  // let setup = now - inputs.1;
  // println!("in the sandbox, the struct is {:?}", inputs);

  let grades = inputs;
  let grades: Vec<[f64; 2]> = grades
      .into_iter()
      .map(|g| [g.0.clone().and_utc().timestamp() as f64, g.1 as f64])
      .collect();
  let array: Array2<f64> = Array2::from(grades);
  let (x, y) = (
      array.slice(s![.., 0..1]).to_owned(),
      array.column(1).to_owned(),
  );

  let dataset: Dataset<f64, f64, Dim<[usize; 1]>> =
      Dataset::new(x, y).with_feature_names(vec!["x", "y"]);

  // Train the model.
  let lin_reg = LinearRegression::new();
  let model = lin_reg.fit(&dataset).unwrap();

  // START TIMER (end in bin)
  // let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;
  let a = now.elapsed().as_nanos() as u64;
  (a, model, 0)
}

// #[AlohomoraSandbox()]
// pub fn encrypt_password_with_recipients(inputs: (String, Vec<String>)) -> Result<String, Error> {
//   let password_plain_text: String = inputs.0;
//   let recipients: Vec<String> = inputs.1;
//   let mut encrypt_buffer = Vec::new();

//   let input_buffer: &[u8] = password_plain_text.as_str().as_bytes();
//   let encrypt_buffer: &mut Vec<u8> = &mut encrypt_buffer;
//   let public_keys: Vec<Box<dyn Recipient + Send>> = recipients
//       .into_iter()
//       .map(|recipient: String| {
//           //TODO: No unwrap
//           println!("recipient is {:?}", recipient.as_str().clone());
//           Box::new(age::x25519::Recipient::from_str(recipient.as_str()).unwrap()) as _
//       })
//       .collect();

//   let encryptor_option = age::Encryptor::with_recipients(public_keys);

//   if let Some(encryptor) = encryptor_option {
//       let mut encrypt_writer = encryptor
//           .wrap_output(encrypt_buffer.compat_mut());

//       encrypt_writer.write_all(input_buffer);

//       encrypt_writer.flush();

//       encrypt_writer.close();

//       Ok(base64.encode(encrypt_buffer))
//   } else {
//       Err()
//   }
// }

// #[AlohomoraSandbox()]
// pub fn gen_pub_key(inputs: Vec<u8>) -> Vec<u8> {
//   println!("reached");
//   let key = age::x25519::Identity::generate();
//   println!("reached");
//   let pubkey = key.to_public();
//   println!("reached");
//   let plaintext: &[u8] = &inputs;
//   let encrypted = {
//     let encryptor = age::Encryptor::with_recipients(vec![Box::new(pubkey)])
//         .expect("we provided a recipient");

//     let mut encrypted = vec![];
//     let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
//     writer.write_all(plaintext).unwrap();
//     writer.finish().unwrap();

//     encrypted
//   };  
//   println!("reached");

//   // ... and decrypt the obtained ciphertext to the plaintext again.
//   let decrypted = {
//       let decryptor = match age::Decryptor::new(&encrypted[..]).unwrap() {
//           age::Decryptor::Recipients(d) => d,
//           _ => unreachable!(),
//       };

//       let mut decrypted = vec![];
//       let mut reader = decryptor.decrypt(iter::once(&key as &dyn age::Identity)).unwrap();
//       let _ = reader.read_to_end(&mut decrypted);

//       decrypted
//   };
//   decrypted
// }