use alohomora_derive::AlohomoraSandbox;

use chrono::naive::NaiveDateTime;
use chrono::Utc;
use linfa::dataset::Dataset;
use linfa::prelude::*;
use linfa_linear::{FittedLinearRegression, LinearRegression};
use ndarray::prelude::*;
use sha2::{Digest, Sha256};


// Sandbox functions.
#[AlohomoraSandbox()]
pub fn hash(inputs: (String, String, u64)) -> (u64, String, u64) {
  // END TIMER (start in bin)
  let now = Utc::now().timestamp_nanos_opt().unwrap() as u64;
  let setup = now - inputs.2;

  let mut hasher = Sha256::new();
  hasher.update(&inputs.0);
  hasher.update(&inputs.1);
  let key = format!("{:x}", hasher.finalize());

  // START TIMER (end in bin)
  let now = Utc::now().timestamp_nanos_opt().unwrap() as u64;
  (setup, key, now)
}

#[AlohomoraSandbox()]
pub fn train(inputs: (Vec<(NaiveDateTime, u64)>, u64)) -> (u64, FittedLinearRegression<f64>, u64) {
  // END TIMER (start in bin)
  let now = Utc::now().timestamp_nanos_opt().unwrap() as u64;
  let setup = now - inputs.1;

  let grades = inputs.0;
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
  let now = Utc::now().timestamp_nanos_opt().unwrap() as u64;
  (setup, model, now)
}
