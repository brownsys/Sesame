use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use lazy_static::lazy_static;
use linfa::dataset::Dataset;
use linfa::prelude::*;
use linfa_linear::{FittedLinearRegression, LinearRegression};
use ndarray::prelude::*;
use rocket::State;

use crate::apikey::ApiKey;
use bbox::context::Context;
use bbox::bbox::{BBox, sandbox_combine, sandbox_execute, MagicUnbox, MagicUnboxEnum};
use bbox::rocket::{BBoxForm, BBoxTemplate};
use bbox_derive::{get, post, BBoxRender, FromBBoxForm};
use bbox::policy::NoPolicy; //{AnyPolicy, NoPolicy, PolicyAnd, SchemaPolicy};


use crate::backend::MySqlBackend;
use crate::policies::ContextData;

pub struct FittedLinRegWrapper {
    x: FittedLinearRegression<f64>
}

//TODO: Derive once MagicUnbox is derivable>
impl MagicUnbox for FittedLinRegWrapper {
    type Out = FittedLinRegWrapper; 
    fn to_enum(self) -> MagicUnboxEnum {
        MagicUnboxEnum::Value(Box::new(self))
    }
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
        match e {
            MagicUnboxEnum::Value(v) => match v.downcast() {
                Ok(v) => Ok(*v),
                _ => Err(()),
            }
            _ => Err(()),
        }
    }
}

lazy_static! {
    static ref MODEL: Arc<Mutex<Option<BBox<FittedLinearRegression<f64>, NoPolicy>>>> =
        Arc::new(Mutex::new(Option::None));
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
    let res = bg.prep_exec("SELECT submitted_at, grade FROM answers", vec![]);
    drop(bg);

    let train = |res: Vec<Vec<mysql::Value>>| {
        // Create the dataset.
        let grades: Vec<[f64; 2]> = res
            .into_iter()
            .map(|r| {
                [
                    mysql::from_value::<NaiveDateTime>(r[0].clone()).timestamp() as f64,
                    mysql::from_value::<u64>(r[1].clone()) as f64,
                ]
            })
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
    };

    let new_model = sandbox_execute(res, train);
    let mut model_ref = MODEL.lock().unwrap();
    *model_ref = Some(new_model);
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
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let ctx = PredictContext {
        lec_id: num,
        parent: "layout".into(),
    };

    BBoxTemplate::render("predict", &ctx, &context)
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct PredictGradeForm {
    time: BBox<String, NoPolicy>,
}

#[derive(BBoxRender)]
struct PredictGradeContext {
    lec_id: BBox<u8, NoPolicy>,
    time: BBox<String, NoPolicy>,
    grade: BBox<f64, NoPolicy>,
    parent: String,
}

#[post("/predict_grade/<num>", data = "<data>")]
pub(crate) fn predict_grade(
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<PredictGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    // Train model if it doesnt exist.
    if !model_exists() {
        train_and_store(backend);
    }

    // Evaluate model.
    let lock = MODEL.lock().unwrap();
    let model = lock.as_ref().unwrap().as_ref().clone(); 
    let time = data.time.as_ref().clone();
    let grade = sandbox_combine(time, model, |time, model| {
        let time = NaiveDateTime::parse_from_str(time.as_str(), "%Y-%m-%d %H:%M:%S");
        model.params()[0] * (time.unwrap().timestamp() as f64) + model.intercept()
    });

    let ctx = PredictGradeContext {
        lec_id: num,
        time: data.time.clone(),
        grade: grade,
        parent: "layout".into(),
    };
    BBoxTemplate::render("predictgrade", &ctx, &context)
}
