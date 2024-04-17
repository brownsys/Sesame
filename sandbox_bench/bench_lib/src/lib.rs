use alohomora_derive::AlohomoraSandbox;

use chrono::naive::NaiveDateTime;
use linfa::dataset::Dataset;
use linfa::prelude::*;
use linfa_linear::{FittedLinearRegression, LinearRegression};
use ndarray::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Argument type for sandboxes.
#[derive(Serialize, Deserialize)]
pub struct Numbers {
    pub a: u32,
    pub b: u32,
}

// Sandbox functions.
#[AlohomoraSandbox()]
pub fn add_numbers(n: Numbers) -> u32 {
    // END TIMER
    n.a + n.b
    // START TIMER
}

#[AlohomoraSandbox()]
pub fn mult_numbers(n: Numbers) -> u32 {
    n.a * n.b
}

#[AlohomoraSandbox()]
pub fn hash(inputs: (String, String)) -> String {
    // END TIMER (start in bin)
    let mut hasher = Sha256::new();
    hasher.update(&inputs.0);
    hasher.update(&inputs.1);
    format!("{:x}", hasher.finalize())
    // START TIMER (end in bin)
}

#[AlohomoraSandbox()]
pub fn train(grades: Vec<(NaiveDateTime, u64)>) -> FittedLinearRegression<f64> {
    // END TIMER (start in bin)
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
    model
    // START TIMER (end in bin)
}
