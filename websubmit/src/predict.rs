use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use lazy_static::lazy_static;

use linfa::dataset::Dataset;
use linfa::prelude::*;
use linfa_linear::{FittedLinearRegression, LinearRegression};
use ndarray::prelude::*;

use rocket::form::{Form, FromForm};
use rocket::{get, post, State};
use rocket_dyn_templates::Template;

use mysql::from_value;

use serde::Serialize;

use crate::backend::MySqlBackend;

lazy_static! {
    static ref MODEL: Arc<Mutex<Option<FittedLinearRegression<f64>>>> = Arc::new(Mutex::new(None));
}

pub fn train(grades: Vec<(NaiveDateTime, u64)>) -> FittedLinearRegression<f64> {
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
}

pub fn evaluate_model(inputs: (String, FittedLinearRegression<f64>)) -> f64 {
    let time = NaiveDateTime::parse_from_str(inputs.0.as_str(), "%Y-%m-%d %H:%M:%S");
    inputs.1.params()[0] * (time.unwrap().and_utc().timestamp() as f64) + inputs.1.intercept()
}

pub(crate) fn model_exists() -> bool {
    let model = MODEL.lock().unwrap();
    match *model {
        None => false,
        _ => true,
    }
}

pub(crate) fn train_and_store(backend: &State<Arc<Mutex<MySqlBackend>>>) {
    println!("Re-training the model and saving it globally...");
    // Get data from database.
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * FROM ml_training WHERE consent = 1", vec![]);
    drop(bg);

    let grades: Vec<(NaiveDateTime, u64)> = res
        .into_iter()
        .map(|r| (from_value(r[1].clone()), from_value(r[0].clone())))
        .collect();

    let new_model = train(grades);
    let mut model_ref = MODEL.lock().unwrap();
    *model_ref = Some(new_model);
}

#[derive(Serialize)]
struct PredictContext {
    lec_id: u8,
    parent: String,
}

#[get("/<num>")]
pub(crate) fn predict(num: u8, _backend: &State<Arc<Mutex<MySqlBackend>>>) -> Template {
    let ctx = PredictContext {
        lec_id: num,
        parent: "layout".into(),
    };

    Template::render("predict", &ctx)
}

#[derive(Debug, FromForm)]
pub(crate) struct PredictGradeForm {
    time: String,
}

#[derive(Serialize)]
struct PredictGradeContext {
    lec_id: u8,
    time: String,
    grade: f64,
    parent: String,
}

#[post("/predict_grade/<num>", data = "<data>")]
pub(crate) fn predict_grade(
    num: u8,
    data: Form<PredictGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    // Train model if it doesn't exist.
    if !model_exists() {
        train_and_store(backend);
    }

    // Evaluate model.
    let time = data.time.clone();
    let model = MODEL.lock().unwrap().as_ref().unwrap().clone();
    let grade = evaluate_model((time, model));

    let ctx = PredictGradeContext {
        lec_id: num,
        time: data.time.clone(),
        grade: grade,
        parent: "layout".into(),
    };
    Template::render("predictgrade", &ctx)
}
