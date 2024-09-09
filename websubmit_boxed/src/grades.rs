use std::sync::{Arc, Mutex};

use alohomora::fold::fold;
use chrono::naive::NaiveDateTime;
use rocket::State;

use alohomora::bbox::{BBox, BBoxRender};
use alohomora::context::Context;
use alohomora::db::from_value;
use alohomora::policy::NoPolicy;
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{get, post, BBoxForm, BBoxRedirect, BBoxTemplate, FromBBoxForm};

use crate::admin::Admin;
use crate::backend::MySqlBackend;
use crate::policies::{AnswerAccessPolicy, ContextData};
use crate::questions::LectureAnswer;
use crate::questions::LectureAnswersContext;

#[get("/<num>")]
pub(crate) fn grades(
    _adm: Admin,
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let key = num.clone().into_bbox::<u64, NoPolicy>();

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM answers WHERE lec = ?",
        (key,),
        context.clone(),
    );
    drop(bg);

    let answers: Vec<LectureAnswer> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()).unwrap(),
            user: from_value(r[0].clone()).unwrap(),
            answer: from_value(r[3].clone()).unwrap(),
            time: from_value(r[4].clone())
                .unwrap()
                .into_ppr(PrivacyPureRegion::new(|v: NaiveDateTime| {
                    v.format("%Y-%m-%d %H:%M:%S").to_string()
                })),
            grade: from_value(r[5].clone()).unwrap(),
        })
        .collect();

    let outer_box_answers: BBox<Vec<crate::questions::LectureAnswerOut>, AnswerAccessPolicy> = fold(answers)
        .unwrap()
        .specialize_policy::<AnswerAccessPolicy>()
        .unwrap();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: outer_box_answers,
        parent: "layout".into(),
    };

    BBoxTemplate::render("grades", &ctx, context)
}

#[derive(BBoxRender)]
struct GradeEditContext {
    answer: BBox<String, AnswerAccessPolicy>,
    grade: BBox<u64, AnswerAccessPolicy>,
    lec_id: BBox<u8, NoPolicy>,
    lec_qnum: BBox<u8, NoPolicy>,
    parent: String,
    user: BBox<String, NoPolicy>,
}

#[get("/<user>/<num>/<qnum>")]
pub(crate) fn editg(
    _adm: Admin,
    user: BBox<String, NoPolicy>,
    num: BBox<u8, NoPolicy>,
    qnum: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT answer, grade FROM answers WHERE email = ? AND lec = ? AND q = ?",
        (
            user.clone(),
            num.clone().into_bbox::<u64, NoPolicy>(),
            qnum.clone().into_bbox::<u64, NoPolicy>(),
        ),
        context.clone(),
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

    BBoxTemplate::render("gradeedit", &ctx, context)
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct EditGradeForm {
    grade: BBox<u64, NoPolicy>,
}

#[post("/editg/<user>/<num>/<qnum>", data = "<data>")]
pub(crate) fn editg_submit(
    _adm: Admin,
    user: BBox<String, NoPolicy>,
    num: BBox<u8, NoPolicy>,
    qnum: BBox<u8, NoPolicy>,
    data: BBoxForm<EditGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxRedirect {
    let mut bg = backend.lock().unwrap();

    bg.prep_exec(
        "UPDATE answers SET grade = ? WHERE email = ? AND lec = ? AND q = ?",
        (data.grade.clone(), user, num.clone(), qnum),
        context.clone(),
    );
    drop(bg);

    // Re-train prediction model given new grade submission.
    // train_and_store(backend, context.clone());

    BBoxRedirect::to("/grades/{}", (&num,), context)
}
