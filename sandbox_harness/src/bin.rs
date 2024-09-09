extern crate alohomora;
extern crate websubmit_boxed_sandboxes;

use std::fs;
use std::time::{Duration, Instant};
use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::sandbox::execute_sandbox;

use websubmit_boxed_sandboxes::{train, hash};

use chrono::naive::NaiveDateTime;
use chrono::DateTime;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

fn hash_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64, u64)> {
    (1..iters + 1).map(|_i| {
        // Generate random email and secret.
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

        // Execute hash in sandbox, recording the different time stamps.
        let output = execute_sandbox::<hash, _, _>((email, secret));
        (
            to_micro(output.total),
            to_micro(output.serialize),
            to_micro(output.setup),
            to_micro(output.function),
            to_micro(output.teardown),
            to_micro(output.deserialize),
            to_micro(output.fold),
        )
    }).collect()
}

fn hash_baseline_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64, u64)> {
    (1..iters + 1).map(|_i| {
        let email: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let secret: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        // // START TIMER (end inside hash)
        let timer = Instant::now();
        let output = websubmit_boxed_sandboxes::hash((email, secret));
        let time = to_micro(timer.elapsed());

        // Print to ensure compiler does not optimize this away.
        println!("{}", output);

        (time, 0, 0, time, 0, 0, 0)
    }).collect()
}

fn train_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64, u64)> {
    type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
    type BBoxGrade = BBox<u64, NoPolicy>;

    (1..iters + 1).map(|_i| {
        let num_grades = 5000; // 5k for benchmarks
        let mut rng = thread_rng();
        let grades: Vec<(BBoxTime, BBoxGrade)> = (1..num_grades + 1).map(|_j| {
            let submitted_at: i64 = rng.gen_range(0..1e15 as i64);
            let submitted_at = DateTime::from_timestamp_nanos(submitted_at).naive_utc();
            let submitted_at = BBox::new(submitted_at, NoPolicy {});
            let grade: u64 = rng.gen_range(0..=100);
            let grade = BBox::new(grade, NoPolicy {});
            (submitted_at, grade)
        }).collect();

        let output = execute_sandbox::<train,_,_>(grades);
        println!("{:?}", output.ret);
        (
            to_micro(output.total),
            to_micro(output.serialize),
            to_micro(output.setup),
            to_micro(output.function),
            to_micro(output.teardown),
            to_micro(output.deserialize),
            to_micro(output.fold),
        )
    }).collect()
}

fn train_baseline_bench(iters: u64) -> Vec<(u64, u64, u64, u64, u64, u64, u64)> {
    (1..iters + 1).map(|_i| {
        let num_grades = 5000; // 5k for benchmarks
        let mut rng = thread_rng();
        let grades: Vec<(NaiveDateTime, u64)> = (1..num_grades + 1).map(|_j| {
            let submitted_at: i64 = rng.gen_range(0..1e15 as i64);
            let submitted_at = DateTime::from_timestamp_nanos(submitted_at).naive_utc();
            let grade: u64 = rng.gen_range(0..=100);
            (submitted_at, grade)
        }).collect();

        let timer = Instant::now();
        let output = websubmit_boxed_sandboxes::train(grades);
        let time = to_micro(timer.elapsed());

        // Print to ensure compiler does not optimize this away.
        println!("{:?}", output);

        (time, 0, 0, time, 0, 0, 0)
    }).collect()
}

fn to_micro(duration: Duration) -> u64 {
    (duration.as_secs() * 1000000) + (duration.as_nanos() as u64 / 1000)
}
fn write_stats(name: String, data: Vec<(u64, u64, u64, u64, u64, u64, u64)>) {
    fs::create_dir_all("results/").unwrap();
    fs::write(
      format!("results/{}.json", name),
      serde_json::to_string_pretty(&data).unwrap(),
    )
    .unwrap();
}

// Runs hashing and training benchmarks, outputting their results to the 'results/' directory.
fn run_benchmarks(){
  println!("running benches");
  let hash_res = hash_bench(1000);
  write_stats("hash".to_string(), hash_res);

  let train_res = train_bench(500);
  write_stats("train".to_string(), train_res);

  let hash_baseline_res = hash_baseline_bench(10);
  write_stats("hash_baseline".to_string(), hash_baseline_res);

  let train_baseline_res = train_baseline_bench(500);
  write_stats("train_baseline".to_string(), train_baseline_res);
}

fn main() {
  // BENCHMARKING
  run_benchmarks();
}
