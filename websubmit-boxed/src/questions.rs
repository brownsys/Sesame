use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Local;
use chrono::naive::NaiveDateTime;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use serde::Serialize;

use bbox::{BBox, BBoxRender};
use bbox::context::Context;
use bbox_derive::BBoxRender;
use bbox::db::{from_value, from_value_or_null};

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;
use crate::helpers::{JoinIdx, left_join};
use crate::policies::ContextData;


#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<u64, BBox<String>>,
}

#[derive(BBoxRender, Clone)]
pub(crate) struct LectureQuestion {
    pub id: BBox<u64>,
    pub prompt: BBox<String>,
    pub answer: BBox<Option<String>>,
}

#[derive(BBoxRender)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: BBox<u8>,
    pub questions: Vec<LectureQuestion>,
    pub parent: String,
}

#[derive(BBoxRender)]
pub struct LectureAnswer {
    pub id: BBox<u64>,
    pub user: BBox<String>,
    pub answer: BBox<String>,
    pub time: BBox<String>,
    pub grade: BBox<u64>,
}

#[derive(BBoxRender)]
pub struct LectureAnswersContext {
    pub lec_id: BBox<u8>,
    pub answers: Vec<LectureAnswer>,
    pub parent: String,
}

#[derive(BBoxRender)]
struct LectureListEntry {
    id: BBox<u64>,
    label: BBox<String>,
    num_qs: BBox<u64>,
    num_answered: u64,
}

#[derive(BBoxRender)]
struct LectureListContext {
    admin: BBox<bool>,
    lectures: Vec<LectureListEntry>,
    parent: String,
}

#[get("/")]
pub(crate) fn leclist(
    apikey: ApiKey,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ApiKey, ContextData>
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
        vec![],
    );
    drop(bg);

    // TODO(babman): pure sandbox.
    let admin: BBox<bool> = apikey.user.sandbox_execute(|email| config.admins.contains(email));
    let lecs: Vec<LectureListEntry> = res
        .into_iter()
        .map(|r| LectureListEntry {
            id: from_value(r[0].clone()),
            label: from_value(r[1].clone()),
            num_qs: r[2].sandbox_execute(|v| if *v == mysql::Value::NULL { 0u64 } else { mysql::from_value(v.clone()) }),
            num_answered: 0u64,
        })
        .collect();

    let ctx = LectureListContext {
        admin,
        lectures: lecs,
        parent: "layout".into(),
    };

    bbox::render("leclist", &ctx, &context).unwrap()
}

#[get("/<num>")]
pub(crate) fn answers(
    num: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>
) -> Template {
    let mut bg = backend.lock().unwrap();
    let key = num.into2::<u64>();
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
        answers,
        parent: "layout".into(),
    };

    bbox::render("answers", &ctx, &context).unwrap()
}

#[get("/<num>")]
pub(crate) fn questions(
    apikey: ApiKey,
    num: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>
) -> Template {
    use std::collections::HashMap;

    let mut bg = backend.lock().unwrap();
    let key = num.into2::<u64>();

    let answers_result = bg.prep_exec(
        "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
        vec![key.clone().into(), apikey.user.into()],
    );
    let questions_result = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ?",
        vec![key.into()]);
    drop(bg);

    let questions = bbox::sandbox_combine(questions_result, answers_result, |questions_res: Vec<Vec<mysql::Value>>, answers_res: Vec<Vec<mysql::Value>>| {
        let mut questions = left_join(questions_res, answers_res, 1, 2, vec![JoinIdx::Left(1), JoinIdx::Left(2), JoinIdx::Right(3)]);
        questions.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        questions
    });

    let questions: Vec<BBox<Vec<mysql::Value>>> = questions.into();
    let questions = questions
        .into_iter()
        .map(|r| {
          let r: Vec<BBox<mysql::Value>> = r.into();
          LectureQuestion {
            id: from_value(r[0].clone()),
            prompt: from_value(r[1].clone()),
            answer: from_value_or_null(r[2].clone())
          }
        })
        .collect();
    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: questions,
        parent: "layout".into(),
    };

    bbox::render("questions", &ctx, &context).unwrap()
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: BBox<u8>,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ApiKey, ContextData>
) -> Redirect {
    let num: BBox<u64> = num.m_into2();
    let ts: mysql::Value = Local::now().naive_local().into();
    let grade: mysql::Value = 0.into();

    let mut bg = backend.lock().unwrap();
    for (id, answer) in &data.answers {
        bg.replace("answers", vec![
            apikey.user.clone().into(),
            num.clone().into(),
            (*id).into(),
            answer.into(),
            ts.clone().into(),
            grade.clone().into(),
        ]);
    }

    // TODO(babman): the email context..
    let answer_log = format!(
        "{}",
        data.answers
            .iter()
            .map(|(i, t)| format!("Question {}:\n{}", i, t.unbox(&context)))
            .collect::<Vec<_>>()
            .join("\n-----\n")
    );
    if config.send_emails {
        let recipients = if *num.unbox(&context) < 90 {
            config.staff.clone()
        } else {
            config.admins.clone()
        };

        email::send(
            bg.log.clone(),
            apikey.user.unbox(&context).clone(),
            recipients,
            format!("{} meeting {} questions", config.class, *num.unbox(&context)),
            answer_log,
        )
            .expect("failed to send email");
    }
    drop(bg);

    bbox::redirect("/leclist", vec![])
}
