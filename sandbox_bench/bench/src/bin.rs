#![feature(allocator_api)]
extern crate alohomora;
extern crate bench_lib;


use std::convert::TryInto;
use std::{fs, thread};
// use std::vec;

use alohomora::bbox::BBox;
use alohomora::policy::{AnyPolicy, NoPolicy};
use alohomora::sandbox::{AlohomoraSandbox, FinalSandboxOut, SandboxInstance};

use alohomora::AlohomoraType;
// use bench_lib::{hash, train};
use bench_lib::{stringy, train, train2, hash};

use chrono::naive::NaiveDateTime;
use chrono::Utc;
use linfa_linear::FittedLinearRegression;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

fn hash_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
  (1..iters + 1).map(|_i| {
    let email: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    let email = BBox::new(email, NoPolicy {});
    let secret: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect();
    let secret = BBox::new(secret, NoPolicy {});

    // START TIMER (end inside hash)
    let now = Utc::now();
    let now = now.timestamp_nanos_opt().unwrap() as u64;

    type Out = (u64, String, u64);
    let output = SandboxInstance::copy_and_execute::<hash, _, _>((email, secret, BBox::new(now, NoPolicy {})));
    // let output = execute_sandbox::<hash, _, _>(); // TODO (allenaby) why does this work if now doesn't have bbox, because impl Alohomora type for u64?
    
    // END TIMER (start inside hash)
    let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

    let output = output.specialize_policy::<NoPolicy>().unwrap();
    let output: Out = output.discard_box();

    // println!("output {:?}", output);

    // let serialize = output.0 - setup; // output.0 -> time in function
    // let deserialize = (end - output.2) - teardown;
    let total = end - now;
    // let function = total - output.0 - (end - output.2);
    let function = output.0;
    // println!("final hash {}", output.1);
    (total, 0, 0, function, 0, 0) // <--- key for results
  }).collect()
}

fn hash_baseline_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
  (1..iters + 1).map(|_i| {
    let email: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    // let email = BBox::new(email, NoPolicy {});
    let secret: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect();
    // let secret = BBox::new(secret, NoPolicy {});

    // // START TIMER (end inside hash)
    // let now = Utc::now();
    // let now = now.timestamp_nanos_opt().unwrap() as u64;

    let output = bench_lib::hash((email, secret, 0));

    // type Out = FinalSandboxOut<(u64, String, u64)>;
    // let output = execute_sandbox::<hash, _, _>((email, secret, BBox::new(now, NoPolicy {}))); // TODO (allenaby) why does this work if now doesn't have bbox, because impl Alohomora type for u64?
    
    // // END TIMER (start inside hash)
    // let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

    // let output = output.specialize_policy::<NoPolicy>().unwrap();
    // let output: Out = output.discard_box();
    // let setup = output.setup;
    // let teardown = output.teardown;
    // let output = output.result;


    // let serialize = output.0 - setup;
    // let deserialize = (end - output.2) - teardown;
    // let total = end - now;
    // let function = total - output.0 - (end - output.2);
    (output.0, 0u64, 0u64, 0u64, 0u64, 0u64)
  }).collect()
}

#[derive(Debug)]
pub struct Grandparent {
    pub cookies_baked: u32,
    pub pickleball_rank: u32,
    pub height: f64,
    pub favorite_kid: *mut Parent,
}

#[derive(Debug, Clone)]
pub struct Parent {
    pub cookouts_held: u32,
    pub hours_at_work: u32,
    pub height: f64,
    pub favorite_kid: *mut Baby,
}

#[derive(Debug, Clone)]
pub struct Baby {
    pub goos_gaad: u32,
    pub iq: u32,
    pub height: f64,
}

fn train_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
  type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
  type BBoxGrade = BBox<u64, NoPolicy>;

  (1..iters + 1).map(|_i| {
    let num_grades = 5000; // 5k for benchmarks
    let mut rng = rand::thread_rng();
    let mut grades: Vec<(BBoxTime, BBoxGrade)> = (1..num_grades + 1).map(|_j| {
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

    // let mut test_grades: Vec<(f64, u64)> = vec![(0.1, 1), (1.2, 2), (2.3, 3), (1003.1, 591), (0.0, 0), (0.131, 1000),(0.1, 1), (1.2, 2), (2.3, 3), (1003.1, 591), (0.0, 0), (0.131, 1000)];
    // let mut test_grades2 = vec![(BBox::new(NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(3, NoPolicy::new())), (BBox::new(NaiveDateTime::parse_from_str("2010-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(10, NoPolicy::new()))];

    // let bbox = BBox::new(mimi_ptr, NoPolicy::new());
    type Out = (usize, (), usize);
    let output = SandboxInstance::copy_and_execute::<train,_,_>(grades.clone());
    // let output = execute_sandbox::<train, _, _>(grades);
    // let output = BBox::new(0, NoPolicy{});

    // END TIMER (start inside hash)
    let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

    let output = output.specialize_policy::<NoPolicy>().unwrap();
    let output: Out = output.discard_box();
    let setup = 0;
    let teardown = 0;
    // let output = 0;

    // let a = speedy_fold::<_, _, _>(grades);

    // let serialize = output.0 - setup; // output.0 -> time in function
    // let deserialize = (end - output.2) - teardown;
    let total = end - now;
    // let function = total - output.0 - (end - output.2);
    let function = output.0;
    (total, 0, setup, function.try_into().unwrap(), teardown.try_into().unwrap(), 0) // <--- key for results
  }).collect()
}

fn train_baseline_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
  // type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
  // type BBoxGrade = BBox<u64, NoPolicy>;

  (1..iters + 1).map(|_i| {
    let num_grades = 5000; // 5k for benchmarks
    let mut rng = rand::thread_rng();
    let grades: Vec<(NaiveDateTime, u64)> = (1..num_grades + 1).map(|_j| {
      let submitted_at: i64 = rng.gen_range(0..1e15 as i64);
      let submitted_at = NaiveDateTime::from_timestamp_nanos(submitted_at).unwrap();
      // let submitted_at = BBox::new(submitted_at, NoPolicy {});
      let grade: u64 = rng.gen_range(0..=100);
      // let grade = BBox::new(grade, NoPolicy {});
      (submitted_at, grade)
    }).collect();

    // // START TIMER (end inside train)
    // let now = Utc::now();
    // let now = now.timestamp_nanos_opt().unwrap() as u64;

    // type Out = FinalSandboxOut<(u64, FittedLinearRegression<f64>, u64)>;
    // let output = execute_sandbox::<train, _, _>((grades, BBox::new(now, NoPolicy {})));
    let output = bench_lib::train(grades);

    // // END TIMER (start inside hash)
    // let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

    // let output = output.specialize_policy::<NoPolicy>().unwrap();
    // let output: Out = output.discard_box();
    // let setup = output.setup;
    // let teardown = output.teardown;
    // let output = output.result;

    // let serialize = output.0 - setup;
    // let deserialize = (end - output.2) - teardown;
    // let total = end - now;
    // let function = total - output.0 - (end - output.2);
    (output.0.try_into().unwrap(), 0u64, 0u64, 0u64, 0u64, 0u64)
  }).collect()
}

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
  // let hash_res = hash_bench(10000);
  // let hash_res = hash_res[0..].to_vec();
  // write_stats("hash".to_string(), hash_res);

  let train_res = train_bench(10000);
  let train_res = train_res[0..].to_vec();
  write_stats("train".to_string(), train_res);

  // for i in 0..10000 {
  //   let t2 = bench_lib::Test2 {
  //     a: 0xdeadbeef
  //   };
  //   let t3 = bench_lib::Test2 {
  //     a: 0xdeadbee2
  //   };
  //   let t4 = bench_lib::Test2 {
  //     a: 0xdeadbee3
  //   };
  //   let t = bench_lib::Test { 
  //     a: 123456,
  //     b: 54321,
  //     c: Box::new(-101012),
  //     // t: t2,
  //     s: String::from("hello there this is a string"),
  //     // ptr: Box::into_raw(Box::new(t3)),
  //     // bx: Box::new(1111),
  //     // ptr2: Box::into_raw(Box::new(t4)),
  //   };

  //   let s = BBox::new(t, NoPolicy::new());
  //   let s_new: BBox<String, NoPolicy> = SandboxInstance::copy_and_execute::<stringy, _, _>(s).specialize_policy().unwrap();
  //   println!("returned--{:?}", s_new.discard_box());
  // }
  

  unsafe{ SandboxInstance::split_info(); }
  // println!("final splits are {:?}", s);

  // let hash_baseline_res = hash_baseline_bench(100);
  // let hash_baseline_res = hash_baseline_res[0..].to_vec();
  // write_stats("hash_baseline".to_string(), hash_baseline_res);

  // let train_baseline_res = train_baseline_bench(100);
  // let train_baseline_res = train_baseline_res[0..].to_vec();
  // write_stats("train_baseline".to_string(), train_baseline_res);
}

// Runs sandboxes with multiple threads to test the sandbox pool.
// fn test_sandbox_pool(){
//   let NUM_THREADS = 100;
//   let mut threads = vec![];

//   // spawn all the testing threads
//   for i in 0..NUM_THREADS {
//       println!("threading -- spawning thread {i}");
//       let t = thread::spawn(||{
//         println!("threading -- hello from thread {:?}", thread::current().id());
//         let hash_res = train_bench(2);
//         println!("threading -- thread {:?} is DONE", thread::current().id());
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

  // BENCHMARK NOTES:
  // first compiling NOOPT commit -
  // for RECREATE - make sure to 

  // SANDBOX POOL TESTING
  // test_sandbox_pool();

  // let mut test_grades = vec![(BBox::new(NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(3, NoPolicy::new())), 
  //                                                              (BBox::new(NaiveDateTime::parse_from_str("2016-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(3, NoPolicy::new())),
  //                                                              (BBox::new(NaiveDateTime::parse_from_str("2017-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(3, NoPolicy::new()))];

  // let out: BBox<FinalSandboxOut<(u64, FittedLinearRegression<f64>, u64)>, NoPolicy> = SandboxInstance::copy_and_execute::<train, _, _>(test_grades).specialize_policy().unwrap();

  // println!("GOT FIRST OUT {:?}", out.discard_box().result);

  // let instance = SandboxInstance::new();
  // let mut test_grades2 = Vec::new_in(instance.alloc());
  // test_grades2.push((BBox::new(NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(3, NoPolicy::new())));
  // test_grades2.push((BBox::new(NaiveDateTime::parse_from_str("2016-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(3, NoPolicy::new())));
  // test_grades2.push((BBox::new(NaiveDateTime::parse_from_str("2017-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy::new()), BBox::new(3, NoPolicy::new())));

  // let out2 = instance.execute::<train, _, _, _>(test_grades2);
  // let out2: BBox<FinalSandboxOut<(u64, FittedLinearRegression<f64>, u64)>, NoPolicy> = out2.specialize_policy().unwrap();

  // println!("GOT SECOND OUT {:?}", out2.discard_box().result);

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
