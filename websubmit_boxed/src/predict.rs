use std::sync::{Arc, Mutex};

use alohomora::db::from_value;
use chrono::naive::NaiveDateTime;
use lazy_static::lazy_static;
use linfa_linear::FittedLinearRegression;
use rocket::State;

use alohomora::context::Context;
use alohomora::bbox::{BBox, BBoxRender};
use alohomora::rocket::{BBoxForm, BBoxTemplate, get, post, FromBBoxForm};
use alohomora::policy::NoPolicy;
use alohomora::sandbox::execute_sandbox;

use crate::backend::MySqlBackend;
use crate::policies::{AnswerAccessPolicy, ContextData};

use websubmit_boxed_sandboxes::train;
use websubmit_boxed_sandboxes::evaluate_model;

lazy_static! {
    static ref MODEL: Arc<Mutex<Option<BBox<FittedLinearRegression<f64>, AnswerAccessPolicy>>>> =
        Arc::new(Mutex::new(None));
}

pub(crate) fn model_exists() -> bool {
    let model = MODEL.lock().unwrap();
    match *model {
        None => false,
        _ => true,
    }
}

pub(crate) fn train_and_store(
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) {
    println!("Re-training the model and saving it globally...");
    // Get data from database.
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM answers", // TODO (allenaby) why does this break if i select submitted_at, grade
        (), 
        context);
    drop(bg);

    type BBoxTime = BBox<NaiveDateTime, AnswerAccessPolicy>;
    type BBoxGrade = BBox<u64, AnswerAccessPolicy>;
    let grades: Vec<(BBoxTime, BBoxGrade)> = res
        .into_iter()
        .map(|r| {
            (
                from_value(r[4].clone()).unwrap(),
                from_value(r[5].clone()).unwrap(),
            )
        })
        .collect();

    let new_model = execute_sandbox::<train, _, _>(grades);
    let mut model_ref = MODEL.lock().unwrap();
    *model_ref = Some(new_model.specialize_policy().unwrap());
}

#[derive(BBoxRender)]
struct PredictContext {
    lec_id: BBox<u8, NoPolicy>,
    parent: String,
}

#[get("/<num>")]
pub(crate) fn predict(
    num: BBox<u8, NoPolicy>,
    _backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let ctx = PredictContext {
        lec_id: num,
        parent: "layout".into(),
    };

    BBoxTemplate::render("predict", &ctx, context)
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct PredictGradeForm {
    time: BBox<String, NoPolicy>,
}

#[derive(BBoxRender)]
struct PredictGradeContext {
    lec_id: BBox<u8, NoPolicy>,
    time: BBox<String, NoPolicy>,
    grade: BBox<f64, AnswerAccessPolicy>,
    parent: String,
}

#[post("/predict_grade/<num>", data = "<data>")]
pub(crate) fn predict_grade(
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<PredictGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    // Train model if it doesn't exist.
    if !model_exists() {
        train_and_store(backend, context.clone());
    }

    // Evaluate model.
    let time = data.time.clone();
    let model = MODEL.lock().unwrap().as_ref().unwrap().clone();
    let grade = execute_sandbox::<evaluate_model, _, _>((time, model));

    let ctx = PredictGradeContext {
        lec_id: num,
        time: data.time.clone(),
        grade: grade.specialize_policy().unwrap(),
        parent: "layout".into(),
    };
    BBoxTemplate::render("predictgrade", &ctx, context)
}
