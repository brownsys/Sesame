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

use bbox::{BBox, BBoxRender, ValueOrBBox};
use bbox_derive::BBoxRender;
use bbox::db::{from_value};

// use crate::admin::Admin;
use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;

#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<u64, BBox<String>>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

#[derive(BBoxRender)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: BBox<u8>,
    pub questions: BBox<Vec<LectureQuestion>>,
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

    bbox::render("leclist", &ctx).unwrap()
}

#[get("/<num>")]
pub(crate) fn answers(
    num: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
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
        answers: answers,
        parent: "layout".into(),
    };

    bbox::render("answers", &ctx).unwrap()
}

#[get("/<num>")]
pub(crate) fn questions(
    apikey: ApiKey,
    num: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
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

    let make_questions = |questions_res: Vec<Vec<mysql::Value>>, answers_res: Vec<Vec<mysql::Value>>| {
        let mut answers = HashMap::new();
        for r in answers_res {
            let id: u64 = mysql::from_value(r[2].clone());
            let atext: String = mysql::from_value(r[3].clone());
            answers.insert(id, atext);
        }

        let mut questions: Vec<_> = questions_res
            .into_iter()
            .map(|r| {
                let id: u64 = mysql::from_value(r[1].clone());
                let answer = answers.get(&id).map(|s| s.to_owned());
                LectureQuestion {
                    id: id,
                    prompt: mysql::from_value(r[2].clone()),
                    answer: answer,
                }
            })
            .collect();
        questions.sort_by(|a, b| a.id.cmp(&b.id));
        questions
    };
    let questions = bbox::sandbox_combine(questions_result, answers_result, make_questions);

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: questions.clone(),
        parent: "layout".into(),
    };

    bbox::render("questions", &ctx).unwrap()
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: BBox<u8>,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
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
            .map(|(i, t)| format!("Question {}:\n{}", i, t.unbox("email")))
            .collect::<Vec<_>>()
            .join("\n-----\n")
    );
    if config.send_emails {
        let recipients = if *num.unbox("email") < 90 {
            config.staff.clone()
        } else {
            config.admins.clone()
        };

        email::send(
            bg.log.clone(),
            apikey.user.unbox("email").clone(),
            recipients,
            format!("{} meeting {} questions", config.class, *num.unbox("email")),
            answer_log,
        )
            .expect("failed to send email");
    }
    drop(bg);

    bbox::redirect("/leclist", vec![])
}
