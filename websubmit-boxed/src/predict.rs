use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use mysql::from_value;
use ndarray::prelude::*;
use rocket::form::{Form, FromForm};
use rocket::State;
use rocket_dyn_templates::Template;

use bbox::{BBox, BBoxRender, ValueOrBBox};
use bbox_derive::BBoxRender;

use crate::backend::{MySqlBackend, Value};


#[derive(BBoxRender)]
struct PredictContext {
    lec_id: BBox<u8>,
    parent: String,
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


#[get("/<num>")]
pub(crate) fn predict(
    num: BBox<u8>,
    _backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let ctx = PredictContext {
        lec_id: num,
        parent: "layout".into(),
    };

    bbox::render("predict", &ctx).unwrap()
}

#[post("/predict_grade/<num>", data = "<data>")]
pub(crate) fn predict_grade(
    num: u8,
    data: Form<PredictGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();

    let num = BBox::new(num);
    let key: BBox<Value> = num.into2::<u64>().into2();
    let res = BBox::internal_new(bg.prep_exec("SELECT submitted_at, grade FROM answers WHERE lec = ?",
                                              vec![key.internal_unbox().clone()],
    ));
    drop(bg);

    // TODO (AllenAby) what granularity makes most sense for sandbox execution?
    let make_dataset = |res: &Vec<Vec<Value>>| {
        let grades: Vec<[f64; 2]> = res
            .into_iter()
            .map(|r| [
                from_value::<NaiveDateTime>(r[0].clone()).timestamp() as f64,
                from_value::<u64>(r[1].clone()) as f64
            ])
            .collect();

        let array: Array2<f64> = Array2::from(grades);

        let (x, y) = (
            array.slice(s![.., 0..1]).to_owned(),
            array.column(1).to_owned()
        );

        let dataset: Dataset<f64, f64, Dim<[usize; 1]>> = Dataset::new(x, y).with_feature_names(vec!["x", "y"]);
        dataset
    };

    let dataset = res.sandbox_execute(make_dataset);

    let model_path = Path::new("model.json");

    // TODO (AllenAby): not sure how to box/unbox when reading/writing from file
    let model = if model_path.exists() {
        println!("Loading the model from a file...");
        let mut file = File::open(model_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_value((&contents).parse().unwrap()).unwrap()
    } else {
        println!("Re-training the model and saving it to disk...");
        let lin_reg = LinearRegression::new();
        // TODO (AllenAby): this is a guess but may not be correct usage of unbox()
        let model = lin_reg.fit(dataset.unbox("grade_prediction")).unwrap();
        let serialized_model = serde_json::to_string(&model).unwrap();
        let mut file = File::create(model_path).unwrap();
        file.write_all(serialized_model.as_ref()).unwrap();
        model
    };

    // TODO (AllenAby) should this be an internal_unbox() or unbox()
    let time = NaiveDateTime::parse_from_str(data.time.internal_unbox().as_str(), "%Y-%m-%d %H:%M:%S");
    let grade = model.params()[0] * (time.unwrap().timestamp() as f64) + model.intercept();
    let grade = BBox::new(grade);

    let ctx = PredictGradeContext {
        lec_id: num,
        time: data.time.clone(),
        grade: grade,
        parent: "layout".into(),
    };

    bbox::render("predictgrade", &ctx).unwrap()
}
