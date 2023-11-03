use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use rocket::State;

use bbox::context::Context;
use bbox::db::from_value;
use bbox::bbox::{BBox};
use bbox::rocket::{BBoxTemplate, BBoxRedirect, BBoxForm};
use bbox_derive::{BBoxRender, FromBBoxForm, get, post};
use bbox::policy::{NoPolicy, AnyPolicy}; //{AnyPolicy, NoPolicy, PolicyAnd, SchemaPolicy};

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::policies::ContextData;
use crate::predict::train_and_store;
use crate::questions::LectureAnswer;
use crate::questions::LectureAnswersContext;

#[get("/<num>")]
pub(crate) fn grades(
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let key = num.clone().into_bbox::<u64>();

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * FROM answers WHERE lec = ?", vec![key.into()]);
    drop(bg);

    let answers: Vec<LectureAnswer> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()).unwrap(),
            user: from_value(r[0].clone()).unwrap(),
            answer: from_value(r[3].clone()).unwrap(),
            time: from_value::<NaiveDateTime, NoPolicy>(r[4].clone()).unwrap()
                .sandbox_execute(|v| v.format("%Y-%m-%d %H:%M:%S").to_string()),
            grade: from_value(r[5].clone()).unwrap(),
        })
        .collect();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers,
        parent: "layout".into(),
    };

    BBoxTemplate::render("grades", &ctx, &context)
}

#[derive(BBoxRender)]
struct GradeEditContext {
    answer: BBox<String, NoPolicy>,
    grade: BBox<u64, NoPolicy>,
    lec_id: BBox<u8, NoPolicy>,
    lec_qnum: BBox<u8, NoPolicy>,
    parent: String,
    user: BBox<String, NoPolicy>,
}

#[get("/<user>/<num>/<qnum>")]
pub(crate) fn editg(
    user: BBox<String, NoPolicy>,
    num: BBox<u8, NoPolicy>,
    qnum: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT answer, grade FROM answers WHERE email = ? AND lec = ? AND q = ?",
        vec![
            user.clone().into(),
            num.clone().into_bbox::<u64>().into(),
            qnum.clone().into_bbox::<u64>().into(),
        ],
    );
    drop(bg);

    let r = &res[0];
    let ctx = GradeEditContext {
        answer: from_value(r[0].clone()).unwrap(),
        grade: from_value(r[1].clone()).unwrap(),
        user: user,
        lec_id: num,
        lec_qnum: qnum,
        parent: "layout".into(),
    };

    BBoxTemplate::render("gradeedit", &ctx, &context)
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct EditGradeForm {
    grade: BBox<u64, NoPolicy>,
}

#[post("/editg/<user>/<num>/<qnum>", data = "<data>")]
pub(crate) fn editg_submit(
    user: BBox<String, NoPolicy>,
    num: BBox<u8, NoPolicy>,
    qnum: BBox<u8, NoPolicy>,
    data: BBoxForm<EditGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> BBoxRedirect {
    let mut bg = backend.lock().unwrap();

    bg.prep_exec(
        "UPDATE answers SET grade = ? WHERE email = ? AND lec = ? AND q = ?",
        vec![
            data.into_inner().grade.into_bbox::<u64>().into(),
            user.into(),
            num.clone().into_bbox::<u64>().into(),
            qnum.into_bbox::<u64>().into(),
        ],
    );
    drop(bg);

    // Re-train prediction model given new grade submission.
    train_and_store(backend);

    BBoxRedirect::to("/grades/{}", vec![&num])
}
