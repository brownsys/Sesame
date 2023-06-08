use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use lazy_static::lazy_static;
use linfa::prelude::*;
use linfa::dataset::Dataset;
use linfa_linear::{LinearRegression, FittedLinearRegression};
use ndarray::prelude::*;
use rocket::form::{Form, FromForm};
use rocket::State;
use rocket_dyn_templates::Template;

use bbox::{BBox, VBox, BBoxRender};
use bbox::context::Context;
use bbox_derive::BBoxRender;
use bbox::db::from_value;
use crate::apikey::ApiKey;

use crate::backend::MySqlBackend;
use crate::policies::ContextData;

lazy_static! {
    static ref MODEL: Arc<Mutex<Option<BBox<FittedLinearRegression<f64>>>>> = Arc::new(Mutex::new(Option::None));
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
  let res = bg.prep_exec(
      "SELECT submitted_at, grade FROM answers",
      vec![],
  );
  drop(bg);

  let train = |res: Vec<Vec<mysql::Value>>| {
    // Create the dataset.
    let grades: Vec<[f64; 2]> = res
        .into_iter()
        .map(|r| [
            mysql::from_value::<NaiveDateTime>(r[0].clone()).timestamp() as f64,
            mysql::from_value::<u64>(r[1].clone()) as f64
        ])
        .collect();

    let array: Array2<f64> = Array2::from(grades);
    let (x, y) = (
        array.slice(s![.., 0..1]).to_owned(),
        array.column(1).to_owned()
    );

    let dataset: Dataset<f64, f64, Dim<[usize; 1]>> = Dataset::new(x, y).with_feature_names(vec!["x", "y"]);

    // Train the model.
    let lin_reg = LinearRegression::new();
    let model = lin_reg.fit(&dataset).unwrap();
    model
  };

  let new_model = bbox::sandbox_execute(res, train);
  let mut model_ref = MODEL.lock().unwrap();
  *model_ref = Some(new_model);
}


#[derive(BBoxRender)]
struct PredictContext {
    lec_id: BBox<u8>,
    parent: String,
}

#[get("/<num>")]
pub(crate) fn predict(
    num: BBox<u8>,
    _backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>
) -> Template {
    let ctx = PredictContext {
        lec_id: num,
        parent: "layout".into(),
    };

    bbox::render("predict", &ctx, &context).unwrap()
}

#[derive(Debug, FromForm)]
pub(crate) struct PredictGradeForm {
    time: BBox<String>,
}

#[derive(BBoxRender)]
struct PredictGradeContext {
    lec_id: BBox<u8>,
    time: BBox<String>,
    grade: BBox<f64>,
    parent: String,
}

#[post("/predict_grade/<num>", data = "<data>")]
pub(crate) fn predict_grade(
    num: BBox<u8>,
    data: Form<PredictGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>
) -> Template {
    // Train model if it doesnt exist.
    if !model_exists() {
      train_and_store(backend);
    }

    // Evaluate model.
    let lock = MODEL.lock().unwrap();
    let model = lock.as_ref().unwrap().as_ref();
    let time = data.time.as_ref();
    let grade = bbox::sandbox_combine(time, model, |time, model| {
       let time = NaiveDateTime::parse_from_str(time.as_str(), "%Y-%m-%d %H:%M:%S");
       model.params()[0] * (time.unwrap().timestamp() as f64) + model.intercept()
    });

    let ctx = PredictGradeContext {
        lec_id: num,
        time: data.time.clone(),
        grade: grade,
        parent: "layout".into(),
    };
    bbox::render("predictgrade", &ctx, &context).unwrap()
}
