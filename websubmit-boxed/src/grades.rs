use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use mysql::from_value;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;

use bbox::{BBox, BBoxRender, ValueOrBBox};
use bbox_derive::BBoxRender;

use crate::backend::{MySqlBackend, Value};
use crate::questions::LectureAnswer;
use crate::questions::LectureAnswersContext;


#[get("/<num>")]
pub(crate) fn grades(
    num: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();

    let key: BBox<Value> = num.into2::<u64>().into2::<Value>();
    let res = BBox::internal_new(bg.prep_exec("SELECT * FROM answers WHERE lec = ?", vec![key.internal_unbox().clone()]));
    drop(bg);

    let answers: BBox<Vec<LectureAnswer>> = res.sandbox_execute(|res: &Vec<Vec<Value>>| {
        let answers: Vec<LectureAnswer> = res
            .into_iter()
            .map(|r| LectureAnswer {
                id: from_value(r[2].clone()),
                user: from_value(r[0].clone()),
                answer: from_value(r[3].clone()),
                time: from_value::<NaiveDateTime>(r[4].clone()).format("%Y-%m-%d %H:%M:%S").to_string(),
                grade: from_value(r[5].clone()),
            })
            .collect();
        answers
    });

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers.clone(),
        parent: "layout".into(),
    };

    bbox::render("grades", &ctx).unwrap()
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
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = BBox::internal_new(bg.prep_exec(
        "SELECT answer, grade FROM answers WHERE email = ? AND lec = ? AND q = ?",
        vec![
            user.into2::<Value>().internal_unbox().clone(),
            num.into2::<u64>().into2::<Value>().internal_unbox().clone(),
            qnum.into2::<u64>().into2::<Value>().internal_unbox().clone(),
        ],
    ));
    drop(bg);

    // TODO (AllenAby) should be inside sandbox?
    let r = &res.internal_unbox()[0];
    let answer: BBox<String> = BBox::new(from_value::<String>(r[0].clone()));
    let grade: BBox<u64> = BBox::new(from_value::<u64>(r[1].clone()));

    let ctx = GradeEditContext {
        answer: answer.clone(),
        grade: grade,
        user: user.clone(),
        lec_id: num,
        lec_qnum: qnum,
        parent: "layout".into(),
    };

    bbox::render("gradeedit", &ctx).unwrap()
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
            data.grade.into2::<u64>().into2::<Value>().internal_unbox().clone(),
            user.into2::<Value>().internal_unbox().clone(),
            num.into2::<u64>().into2::<Value>().internal_unbox().clone(),
            qnum.into2::<u64>().into2::<Value>().internal_unbox().clone(),
        ],
    );
    drop(bg);

    bbox::redirect("/grades/{}", vec![&num])
}
