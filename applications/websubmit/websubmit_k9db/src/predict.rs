use std::sync::{Arc, Mutex};

use alohomora::db::from_value;
use chrono::naive::NaiveDateTime;
use lazy_static::lazy_static;
use linfa_linear::FittedLinearRegression;
use rocket::State;

use alohomora::bbox::{BBox, BBoxRender};
use alohomora::k9db::policies::Consent;
use alohomora::policy::{AnyPolicy, NoPolicy, PolicyOr};
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{get, post, BBoxForm, BBoxTemplate, FromBBoxForm, JsonResponse};
use alohomora::sandbox::execute_sandbox;

use crate::backend::MySqlBackend;
use crate::manage::Manager;
use crate::policies::{Context, ContextData};

use websubmit_boxed_sandboxes::train;

lazy_static! {
    static ref MODEL: Arc<Mutex<Option<BBox<FittedLinearRegression<f64>, AnyPolicy>>>> =
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
    context: Context,
) {
    type Or = PolicyOr<AnyPolicy, AnyPolicy>;
    use alohomora::policy::Policy;
    // println!("Re-training the model and saving it globally...");
    // Get data from database.
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * FROM ml_training WHERE consent = 1", (), context);
    drop(bg);

    type BBoxTime = BBox<NaiveDateTime, Consent>;
    type BBoxGrade = BBox<u64, Consent>;
    let grades: Vec<(BBoxTime, BBoxGrade)> = res
        .into_iter()
        .map(|r| {
            // Read from DB.
            let time = from_value::<NaiveDateTime, Or>(r[1].clone()).unwrap();
            let grade = from_value::<u64, Or>(r[0].clone()).unwrap();
            // Specialize policies.
            let time: BBoxTime = time.specialize_right().specialize_policy().unwrap();
            let grade: BBoxGrade = grade.specialize_right().specialize_policy().unwrap();
            (time, grade)
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
    _manager: Manager,
    num: BBox<u8, NoPolicy>,
    _backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context,
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
    time: String,
    grade: BBox<Vec<f64>, AnyPolicy>,
    parent: String,
}

#[post("/predict_grade/<num>", data = "<data>")]
pub(crate) fn predict_grade(
    // _manager: Manager,
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<PredictGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context,
) -> BBoxTemplate {
    // Train model if it doesn't exist.
    if !model_exists() {
        train_and_store(backend, context.clone());
    }

    // Evaluate model.
    let time_string = data.into_inner().time.discard_box();
    let model = MODEL.lock().unwrap().as_ref().unwrap().clone();
    let grades = model.ppr(PrivacyPureRegion::new(
        |model: &FittedLinearRegression<f64>| {
            time_string
                .split(',')
                .map(|time_string| NaiveDateTime::parse_from_str(time_string, "%Y-%m-%d %H:%M:%S"))
                .map(|time| {
                    model.params()[0] * (time.unwrap().and_utc().timestamp() as f64)
                        + model.intercept()
                })
                .collect()
        },
    ));

    let ctx = PredictGradeContext {
        lec_id: num,
        time: time_string,
        grade: grades.to_owned_policy(),
        parent: "layout".into(),
    };
    BBoxTemplate::render("predictgrade", &ctx, context)
}

#[get("/retrain_model")]
pub(crate) fn retrain_model(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context,
) -> JsonResponse<String, ContextData> {
    train_and_store(backend, context.clone());
    JsonResponse::from(("Successfully retrained the model.".to_owned(), context))
}
