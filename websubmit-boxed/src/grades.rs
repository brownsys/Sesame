use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;

use bbox::{BBox, VBox, BBoxRender};
use bbox::context::Context;
use bbox_derive::BBoxRender;
use bbox::db::from_value;
use crate::apikey::ApiKey;

use crate::backend::MySqlBackend;
use crate::policies::ContextData;
use crate::questions::LectureAnswer;
use crate::questions::LectureAnswersContext;
use crate::predict::train_and_store;

#[get("/<num>")]
pub(crate) fn grades(
    num: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>
) -> Template {
    let key = num.into2::<u64>();

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * FROM answers WHERE lec = ?", vec![key.into()]);
    drop(bg);

    let answers: Vec<LectureAnswer> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()),
            user: from_value(r[0].clone()),
            answer: from_value(r[3].clone()),
            time: from_value::<NaiveDateTime>(r[4].clone()).sandbox_execute(|v| v.format("%Y-%m-%d %H:%M:%S").to_string()),
            grade: from_value(r[5].clone()),
        })
        .collect();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers,
        parent: "layout".into(),
    };

    bbox::render("grades", &ctx, &context).unwrap()
}


#[derive(BBoxRender)]
struct GradeEditContext {
    answer: BBox<String>,
    grade: BBox<u64>,
    lec_id: BBox<u8>,
    lec_qnum: BBox<u8>,
    parent: String,
    user: BBox<String>
}

#[get("/<user>/<num>/<qnum>")]
pub(crate) fn editg(
    user: BBox<String>,
    num: BBox<u8>,
    qnum: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT answer, grade FROM answers WHERE email = ? AND lec = ? AND q = ?",
        vec![
            user.clone().into(),
            num.into2::<u64>().into(),
            qnum.into2::<u64>().into(),
        ]);
    drop(bg);

    let r = &res[0];
    let ctx = GradeEditContext {
        answer: from_value(r[0].clone()),
        grade: from_value(r[1].clone()),
        user: user,
        lec_id: num,
        lec_qnum: qnum,
        parent: "layout".into(),
    };

    bbox::render("gradeedit", &ctx, &context).unwrap()
}

#[derive(Debug, FromForm)]
pub(crate) struct EditGradeForm {
    grade: BBox<u64>,
}

#[post("/editg/<user>/<num>/<qnum>", data = "<data>")]
pub(crate) fn editg_submit(
    user: BBox<String>,
    num: BBox<u8>,
    qnum: BBox<u8>,
    data: Form<EditGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();

    bg.prep_exec(
        "UPDATE answers SET grade = ? WHERE email = ? AND lec = ? AND q = ?",
        vec![
            data.grade.into2::<u64>().into(),
            user.clone().into(),
            num.into2::<u64>().into(),
            qnum.into2::<u64>().into(),
        ]);
    drop(bg);

    // Re-train prediction model given new grade submission.
    train_and_store(backend);   

    bbox::redirect("/grades/{}", vec![&num])
}
