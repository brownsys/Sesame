extern crate alohomora;
extern crate bench_lib;

use std::{fs, thread};
// use std::vec;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::{execute_sandbox, FinalSandboxOut};

// use bench_lib::{hash, train};
use bench_lib::train2;

use chrono::naive::NaiveDateTime;
use chrono::Utc;
use linfa_linear::FittedLinearRegression;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

// fn hash_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
//   (1..iters + 1).map(|_i| {
//     let email: String = thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(30)
//         .map(char::from)
//         .collect();
//     let email = BBox::new(email, NoPolicy {});
//     let secret: String = thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(15)
//         .map(char::from)
//         .collect();
//     let secret = BBox::new(secret, NoPolicy {});

//     // START TIMER (end inside hash)
//     let now = Utc::now();
//     let now = now.timestamp_nanos_opt().unwrap() as u64;

//     type Out = FinalSandboxOut<(u64, String, u64)>;
//     let output = execute_sandbox::<hash, _, _>((email, secret, BBox::new(now, NoPolicy {}))); // TODO (allenaby) why does this work if now doesn't have bbox, because impl Alohomora type for u64?
    
//     // END TIMER (start inside hash)
//     let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

//     let output = output.specialize_policy::<NoPolicy>().unwrap();
//     let output: Out = output.discard_box();
    
//     let setup = output.setup; // output.setup -> how much time it takes to make the sandbox
//     // println!("setup: {:?}", setup);
//     let teardown = output.teardown;
//     // println!("tear: {:?}", teardown);
//     let output = output.result;
//     // println!("output: {:?}", output);

//     // let serialize = output.0 - setup; // output.0 -> time in function
//     // let deserialize = (end - output.2) - teardown;
//     let total = end - now;
//     // let function = total - output.0 - (end - output.2);
//     let function = output.0;
//     // println!("final hash {}", output.1);
//     (total, 0, setup, function, teardown, 0) // <--- key for results
//   }).collect()
// }

// fn hash_baseline_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
//   (1..iters + 1).map(|_i| {
//     let email: String = thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(30)
//         .map(char::from)
//         .collect();
//     // let email = BBox::new(email, NoPolicy {});
//     let secret: String = thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(15)
//         .map(char::from)
//         .collect();
//     // let secret = BBox::new(secret, NoPolicy {});

//     // // START TIMER (end inside hash)
//     // let now = Utc::now();
//     // let now = now.timestamp_nanos_opt().unwrap() as u64;

//     let output = bench_lib::hash((email, secret, 0));

//     // type Out = FinalSandboxOut<(u64, String, u64)>;
//     // let output = execute_sandbox::<hash, _, _>((email, secret, BBox::new(now, NoPolicy {}))); // TODO (allenaby) why does this work if now doesn't have bbox, because impl Alohomora type for u64?
    
//     // // END TIMER (start inside hash)
//     // let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

//     // let output = output.specialize_policy::<NoPolicy>().unwrap();
//     // let output: Out = output.discard_box();
//     // let setup = output.setup;
//     // let teardown = output.teardown;
//     // let output = output.result;


//     // let serialize = output.0 - setup;
//     // let deserialize = (end - output.2) - teardown;
//     // let total = end - now;
//     // let function = total - output.0 - (end - output.2);
//     (output.0, 0u64, 0u64, 0u64, 0u64, 0u64)
//   }).collect()
// }

#[derive(Debug)]
pub struct Grandparent {
    pub cookies_baked: u32,
    pub pickleball_rank: u32,
    pub height: f64,
    pub favorite_kid: *mut Parent,
}

#[derive(Debug)]
pub struct Parent {
    pub cookouts_held: u32,
    pub hours_at_work: u32,
    pub height: f64,
    pub favorite_kid: *mut Baby,
}

#[derive(Debug)]
pub struct Baby {
    pub goos_gaad: u32,
    pub iq: u32,
    pub height: f64,
}

fn train_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
  type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
  type BBoxGrade = BBox<u64, NoPolicy>;

  (1..iters + 1).map(|_i| {
    let num_grades = 5000;
    let mut rng = rand::thread_rng();
    let grades: Vec<(BBoxTime, BBoxGrade)> = (1..num_grades + 1).map(|_j| {
      let submitted_at: i64 = rng.gen_range(0..1e15 as i64);
      let submitted_at = NaiveDateTime::from_timestamp_nanos(submitted_at).unwrap();
      let submitted_at = BBox::new(submitted_at, NoPolicy {});
      let grade: u64 = rng.gen_range(0..=100);
      let grade = BBox::new(grade, NoPolicy {});
      (submitted_at, grade)
    }).collect();

    // START TIMER (end inside train)
    let now = Utc::now();
    let now = now.timestamp_nanos_opt().unwrap() as u64;

    let mut test_grades: Vec<(f64, u64)> = vec![(0.1, 1), (1.2, 2), (2.3, 3), (1003.1, 591), (0.0, 0)];

    let baby = Box::new(Baby {
      goos_gaad: 18,
      iq: 10,
      height: 1.3,
    });
    let baby_ptr = Box::into_raw(baby);
  
    let mom = Box::new(Parent {
      cookouts_held: 3,
      hours_at_work: 1004919491,
      height: 6.2,
      favorite_kid: baby_ptr,
    });
    let mom_ptr = Box::into_raw(mom);
  
    let mimi = Box::new(Grandparent {
      cookies_baked: 13000,
      pickleball_rank: 1,
      height: 5.8,
      favorite_kid: mom_ptr,
    });
    let mimi_ptr = Box::into_raw(mimi);
    
    let test_ptr: &mut Vec<(f64, u64)> = &mut test_grades;
    let ptr = mimi_ptr as *mut Vec<(f64, u64)>;
    let void_ptr = ptr as *mut std::ffi::c_void;
    

    type Out = FinalSandboxOut<(u64, (), u64)>;
    let output = execute_sandbox::<train2, _, _>(void_ptr);

    // END TIMER (start inside hash)
    let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

    let output = output.specialize_policy::<NoPolicy>().unwrap();
    let output: Out = output.discard_box();
    let setup = output.setup;
    let teardown = output.teardown;
    let output = output.result;

    // let serialize = output.0 - setup; // output.0 -> time in function
    // let deserialize = (end - output.2) - teardown;
    let total = end - now;
    // let function = total - output.0 - (end - output.2);
    let function = output.0;
    (total, 0, setup, function, teardown, 0) // <--- key for results
  }).collect()
}

// fn train_baseline_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
//   // type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
//   // type BBoxGrade = BBox<u64, NoPolicy>;

//   (1..iters + 1).map(|_i| {
//     let num_grades = 5000;
//     let mut rng = rand::thread_rng();
//     let grades: Vec<(NaiveDateTime, u64)> = (1..num_grades + 1).map(|_j| {
//       let submitted_at: i64 = rng.gen_range(0..1e15 as i64);
//       let submitted_at = NaiveDateTime::from_timestamp_nanos(submitted_at).unwrap();
//       // let submitted_at = BBox::new(submitted_at, NoPolicy {});
//       let grade: u64 = rng.gen_range(0..=100);
//       // let grade = BBox::new(grade, NoPolicy {});
//       (submitted_at, grade)
//     }).collect();

//     // // START TIMER (end inside train)
//     // let now = Utc::now();
//     // let now = now.timestamp_nanos_opt().unwrap() as u64;

//     // type Out = FinalSandboxOut<(u64, FittedLinearRegression<f64>, u64)>;
//     // let output = execute_sandbox::<train, _, _>((grades, BBox::new(now, NoPolicy {})));
//     let output = bench_lib::train((grades, 0));

//     // // END TIMER (start inside hash)
//     // let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

//     // let output = output.specialize_policy::<NoPolicy>().unwrap();
//     // let output: Out = output.discard_box();
//     // let setup = output.setup;
//     // let teardown = output.teardown;
//     // let output = output.result;

//     // let serialize = output.0 - setup;
//     // let deserialize = (end - output.2) - teardown;
//     // let total = end - now;
//     // let function = total - output.0 - (end - output.2);
//     (output.0, 0u64, 0u64, 0u64, 0u64, 0u64)
//   }).collect()
// }

fn write_stats(name: String, data: Vec<(u64, u64, u64, u64, u64, u64)>) {
  fs::create_dir_all("results/").unwrap();
  fs::write(
      format!("results/{}.json", name),
      serde_json::to_string_pretty(&data).unwrap(),
  )
  .unwrap();
}

// Runs hashing and training benchmarks, outputting their results to the 'results/' directory.
fn run_benchmarks(){
  // let hash_res = hash_bench(100);
  // let hash_res = hash_res[0..].to_vec();
  // write_stats("hash".to_string(), hash_res);

  let train_res = train_bench(1);
  let train_res = train_res[0..].to_vec();
  write_stats("train".to_string(), train_res);

  // let hash_baseline_res = hash_baseline_bench(100);
  // let hash_baseline_res = hash_baseline_res[0..].to_vec();
  // write_stats("hash_baseline".to_string(), hash_baseline_res);

  // let train_baseline_res = train_baseline_bench(100);
  // let train_baseline_res = train_baseline_res[0..].to_vec();
  // write_stats("train_baseline".to_string(), train_baseline_res);
}

// Runs sandboxes with multiple threads to test the sandbox pool.
// fn test_sandbox_pool(){
//   let NUM_THREADS = 4;
//   let mut threads = vec![];

//   // spawn all the testing threads
//   for i in 0..NUM_THREADS {
//       println!("spawning thread {i}");
//       let t = thread::spawn(||{
//         println!("hello from thread {:?}", thread::current().id());
//         let hash_res = hash_bench(2);
//       });
//       threads.push(t);
//   }

//   // join all the testing threads
//   while threads.len() > 0 {
//     let t = threads.remove(0);
//     t.join().unwrap();
//   }
// }

fn main() {
  // BENCHMARKING
  run_benchmarks();
  // TODO:
  // - why is there no setup that takes a longer amount of time?
  // - why are we getting more round numbers than expected or similar times
  // - does a smaller sandbox make this faster?
  // - does the madvise wipe make it proportionally faster?

  // SANDBOX POOL TESTING
  // test_sandbox_pool();

  /*
  let output = execute_sandbox::<global_test, _, _>(BBox::new(String::from(""), NoPolicy {}));
  println!("{}", output.specialize_policy::<NoPolicy>().unwrap().discard_box().result);

  let output = execute_sandbox::<global_test, _, _>(BBox::new(String::from(""), NoPolicy {}));
  println!("{}", output.specialize_policy::<NoPolicy>().unwrap().discard_box().result);


  let output = execute_sandbox::<global_test, _, _>(BBox::new(String::from(""), NoPolicy {}));
  println!("{}", output.specialize_policy::<NoPolicy>().unwrap().discard_box().result);


  let output = execute_sandbox::<global_test, _, _>(BBox::new(String::from(""), NoPolicy {}));
  println!("{}", output.specialize_policy::<NoPolicy>().unwrap().discard_box().result);
  */
  // let y = 3;
  // let password = b"Hello world!";
  // let password = Vec::from(password);
  // let password = BBox::new(password, NoPolicy {});
  // let bbox = execute_sandbox::<gen_pub_key, _, _>(password);
  // let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
  // println!("{:#?}", bbox.discard_box().result);
}
