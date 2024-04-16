use chrono::NaiveDateTime;
use std::sync::{Arc, Mutex};

use mysql::from_value;

use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_dyn_templates::Template;

use serde::Serialize;

use crate::admin::Admin;
use crate::{
    backend::MySqlBackend,
    questions::{LectureAnswer, LectureAnswersContext},
};

#[get("/<num>")]
pub(crate) fn grades(_adm: Admin, num: u8, backend: &State<Arc<Mutex<MySqlBackend>>>) -> Template {
    let key = (num as u64).into();

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * FROM answers WHERE lec = ?", vec![key]);
    drop(bg);

    let answers: Vec<LectureAnswer> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()),
            user: from_value(r[0].clone()),
            answer: from_value(r[3].clone()),
            time: from_value::<NaiveDateTime>(r[4].clone())
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            grade: from_value(r[5].clone()),
        })
        .collect();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers,
        parent: "layout".into(),
    };

    Template::render("grades", &ctx)
}

#[derive(Serialize)]
struct GradeEditContext {
    answer: String,
    grade: u64,
    user: String,
    lec_id: u8,
    lec_qnum: u8,
    parent: String,
}

#[get("/<user>/<num>/<qnum>")]
pub(crate) fn editg(
    _adm: Admin,
    user: String,
    num: u8,
    qnum: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT answer, grade FROM answers WHERE email = ? AND lec = ? AND q = ?",
        vec![
            user.clone().into(),
            (num as u64).into(),
            (qnum as u64).into(),
        ],
    );
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

    Template::render("gradeedit", &ctx)
}

#[derive(Debug, FromForm)]
pub(crate) struct EditGradeForm {
    grade: u64,
}

#[post("/editg/<user>/<num>/<qnum>", data = "<data>")]
pub(crate) fn editg_submit(
    _adm: Admin,
    user: String,
    num: u8,
    qnum: u8,
    data: Form<EditGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();

    bg.prep_exec(
        "UPDATE answers SET grade = ? WHERE email = ? AND lec = ? AND q = ?",
        vec![data.grade.into(), user.into(), num.into(), qnum.into()],
    );
    drop(bg);

    // Re-train prediction model given new grade submission.
    // train_and_store(backend);

    Redirect::to(format!("/grades/{}", num))
}
