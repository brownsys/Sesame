extern crate alohomora;
extern crate bench_lib;

use std::fs;
use std::vec;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::execute_sandbox;

use bench_lib::{hash, train};

use chrono::naive::NaiveDateTime;
use chrono::Utc;
// use linfa_linear::FittedLinearRegression;

fn hash_bench(iters: u64) -> Vec<(i64, i64)> {
  (1..iters).map(|_i| {
    let email = BBox::new("allen_aby@brown.edu".to_string(), NoPolicy {});
    let secret = BBox::new("SECRET".to_string(), NoPolicy {});

    // START TIMER (end inside hash)
    let now = Utc::now();
    let now = now.timestamp_nanos_opt().unwrap();

    let output: BBox<(i64, String, i64), alohomora::policy::AnyPolicy> = execute_sandbox::<hash, _, _>((email, secret, BBox::new(now, NoPolicy {})));
    
    // END TIMER (start inside hash)
    let now = Utc::now().timestamp_nanos_opt().unwrap();

    let output = output.specialize_policy::<NoPolicy>().unwrap();
    let output: (i64, String, i64) = output.discard_box();
    let teardown = now - output.2;
    (output.0, teardown)
  }).collect()
}

fn train_bench(iters: u64) -> Vec<(i64, i64)> {
  type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
  type BBoxGrade = BBox<u64, NoPolicy>;

  (1..iters).map(|i| {
    println!("{}", i);
    let grades: Vec<(BBoxTime, BBoxGrade)> = vec![
      (BBox::new(NaiveDateTime::parse_from_str("2023-03-13 13:40:26", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy {}), BBox::new(90 as u64, NoPolicy {})),
      (BBox::new(NaiveDateTime::parse_from_str("2023-03-09 13:54:05", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy {}), BBox::new(95 as u64, NoPolicy {})),
    ];

    // START TIMER (end inside train)
    let now = Utc::now();
    let now = now.timestamp_nanos_opt().unwrap();
    println!("reached");

    let output = execute_sandbox::<train, _, _>((grades, now));

    // END TIMER (start inside hash)
    let now = Utc::now().timestamp_nanos_opt().unwrap();

    let output = output.specialize_policy::<NoPolicy>().unwrap();
    let output = output.discard_box();
    let teardown = now - output.2;
    (output.0, teardown)
    // let time = NaiveDateTime::parse_from_str("2023-03-13 13:40:50", "%Y-%m-%d %H:%M:%S");
    // let grade = model.into_ppr(PrivacyPureRegion::new(|model: FittedLinearRegression<f64>|
    //   model.params()[0] * (time.unwrap().and_utc().timestamp() as f64) + model.intercept()
    // ));
    // let grade = grade.specialize_policy::<NoPolicy>().unwrap();
  }).collect()
}

fn write_stats(name: String, data: Vec<(i64, i64)>) {
  fs::create_dir_all("benches/").unwrap();
  fs::write(
      format!("benches/{}.json", name),
      serde_json::to_string_pretty(&data).unwrap(),
  )
  .unwrap();
}

fn main() {
  let hash_res = hash_bench(100);
  write_stats("hash".to_string(), hash_res);

  // let train_res = train_bench(100);
  // write_stats("train".to_string(), train_res);
}
