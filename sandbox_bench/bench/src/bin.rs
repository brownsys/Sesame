extern crate alohomora;
extern crate bench_lib;

use std::fs;
use std::vec;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::{execute_sandbox, FinalSandboxOut};

use bench_lib::{hash, train};

use chrono::naive::NaiveDateTime;
use chrono::Utc;
use linfa_linear::FittedLinearRegression;

fn hash_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
  (1..iters + 1).map(|_i| {
    let email = BBox::new("allen_aby@brown.edu".to_string(), NoPolicy {});
    let secret = BBox::new("SECRET".to_string(), NoPolicy {});

    // START TIMER (end inside hash)
    let now = Utc::now();
    let now = now.timestamp_nanos_opt().unwrap() as u64;

    type Out = FinalSandboxOut<(u64, String, u64)>;
    let output = execute_sandbox::<hash, _, _>((email, secret, BBox::new(now, NoPolicy {}))); // TODO (allenaby) why does this work if now doesn't have bbox, because impl Alohomora type for u64?
    
    // END TIMER (start inside hash)
    let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

    let output = output.specialize_policy::<NoPolicy>().unwrap();
    let output: Out = output.discard_box();
    let setup = output.setup;
    let teardown = output.teardown;
    let output = output.result;


    let serialize = output.0 - setup;
    let deserialize = (end - output.2) - teardown;
    let total = end - now;
    let function = total - output.0 - (end - output.2);
    (total, serialize, setup, function, teardown, deserialize)
  }).collect()
}

fn train_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64)> {
  type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
  type BBoxGrade = BBox<u64, NoPolicy>;

  (1..iters + 1).map(|_i| {
    let grades: Vec<(BBoxTime, BBoxGrade)> = vec![
      (BBox::new(NaiveDateTime::parse_from_str("2023-03-13 13:40:26", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy {}), BBox::new(90 as u64, NoPolicy {})),
      (BBox::new(NaiveDateTime::parse_from_str("2023-03-09 13:54:05", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy {}), BBox::new(95 as u64, NoPolicy {})),
    ];

    // START TIMER (end inside train)
    let now = Utc::now();
    let now = now.timestamp_nanos_opt().unwrap() as u64;

    type Out = FinalSandboxOut<(u64, FittedLinearRegression<f64>, u64)>;
    let output = execute_sandbox::<train, _, _>((grades, BBox::new(now, NoPolicy {})));

    // END TIMER (start inside hash)
    let end = Utc::now().timestamp_nanos_opt().unwrap() as u64;

    let output = output.specialize_policy::<NoPolicy>().unwrap();
    let output: Out = output.discard_box();
    let setup = output.setup;
    let teardown = output.teardown;
    let output = output.result;

    let serialize = output.0 - setup;
    let deserialize = (end - output.2) - teardown;
    let total = end - now;
    let function = total - output.0 - (end - output.2);
    (total, serialize, setup, function, teardown, deserialize)
    // let time = NaiveDateTime::parse_from_str("2023-03-13 13:40:50", "%Y-%m-%d %H:%M:%S");
    // let grade = model.into_ppr(PrivacyPureRegion::new(|model: FittedLinearRegression<f64>|
    //   model.params()[0] * (time.unwrap().and_utc().timestamp() as f64) + model.intercept()
    // ));
    // let grade = grade.specialize_policy::<NoPolicy>().unwrap();
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

fn main() {
  let hash_res = hash_bench(100);
  write_stats("hash".to_string(), hash_res);

  let train_res = train_bench(100);
  write_stats("train".to_string(), train_res);
}
