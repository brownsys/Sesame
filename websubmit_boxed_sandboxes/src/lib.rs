// use serde::{Deserialize, Serialize};
use chrono::naive::NaiveDateTime;
use linfa::dataset::Dataset;
use linfa::prelude::*;
use linfa_linear::{FittedLinearRegression, LinearRegression};
// use mysql::*;
use ndarray::prelude::*;

use alohomora_derive::AlohomoraSandbox;

// Sandbox functions.
#[AlohomoraSandbox()]
pub fn hash(inputs: (String, String)) -> String {
    format!("hash{}", inputs.0)
}

// #[AlohomoraSandbox()]
// pub fn train(res: Vec<Vec<mysql::Value>>) -> FittedLinearRegression<f64> {
//     let grades: Vec<[f64; 2]> = res
//         .into_iter()
//         .map(|r| {
//             [
//                 mysql::from_value::<NaiveDateTime>(r[0].clone()).timestamp() as f64,
//                 mysql::from_value::<u64>(r[1].clone()) as f64,
//             ]
//         })
//         .collect();

//     let array: Array2<f64> = Array2::from(grades);
//     let (x, y) = (
//         array.slice(s![.., 0..1]).to_owned(),
//         array.column(1).to_owned(),
//     );

//     let dataset: Dataset<f64, f64, Dim<[usize; 1]>> =
//         Dataset::new(x, y).with_feature_names(vec!["x", "y"]);

//     // Train the model.
//     let lin_reg = LinearRegression::new();
//     let model = lin_reg.fit(&dataset).unwrap();
//     model
// }

#[AlohomoraSandbox()]
pub fn evaluate_model(inputs: (String, FittedLinearRegression<f64>)) -> f64 {
    let time = NaiveDateTime::parse_from_str(inputs.0.as_str(), "%Y-%m-%d %H:%M:%S");
    inputs.1.params()[0] * (time.unwrap().timestamp() as f64) + inputs.1.intercept()
}
