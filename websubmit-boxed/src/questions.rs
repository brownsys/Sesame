use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Local;
use chrono::naive::NaiveDateTime;
use mysql::from_value;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use serde::Serialize;

use bbox::{BBox, BBoxRender, ValueOrBBox};
use bbox_derive::BBoxRender;

// use crate::admin::Admin;
use crate::apikey::ApiKey;
use crate::backend::{MySqlBackend, Value};
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

#[derive(Serialize, Clone)]
pub(crate) struct LectureAnswer {
    pub id: u64,
    pub user: String,
    pub answer: String,
    pub time: String,
    pub grade: u64,
}

#[derive(BBoxRender)]
pub(crate) struct LectureAnswersContext {
    pub lec_id: BBox<u8>,
    pub answers: BBox<Vec<LectureAnswer>>,
    pub parent: String,
}

#[derive(Serialize)]
struct LectureListEntry {
    id: u64,
    label: String,
    num_qs: u64,
    num_answered: u64,
}

#[derive(BBoxRender)]
struct LectureListContext {
    admin: BBox<bool>,
    lectures: BBox<Vec<LectureListEntry>>,
    parent: String,
}

#[get("/")]
pub(crate) fn leclist(
    apikey: ApiKey,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = BBox::internal_new(bg.prep_exec(
        "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
        vec![],
    ));
    drop(bg);

    let admin: BBox<bool> = apikey.user.sandbox_execute(|email| {
        if config.admins.contains(email) {
            true
        } else {
            false
        }
    });

    let lecs: BBox<Vec<LectureListEntry>> = res.sandbox_execute(|res: &Vec<Vec<Value>>| {
        let lecs: Vec<LectureListEntry> = res
            .into_iter()
            .map(|r| LectureListEntry {
                id: from_value(r[0].clone()),
                label: from_value(r[1].clone()),
                num_qs: if r[2] == Value::NULL {
                    0u64
                } else {
                    from_value(r[2].clone())
                },
                num_answered: 0u64,
            })
            .collect();
        lecs
    });

    let ctx = LectureListContext {
        admin,
        lectures: lecs,
        parent: "layout".into(),
    };

    bbox::render("leclist", &ctx).unwrap()
}

#[get("/<num>")]
pub(crate) fn answers(
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let num = BBox::new(num);
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
    let key: BBox<Value> = num.into2::<u64>().into2::<Value>();

    let answers_result = BBox::internal_new(bg.prep_exec(
        "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
        vec![key.internal_unbox().clone(), apikey.user.into2::<Value>().internal_unbox().clone()],
    ));

    let questions_result = BBox::internal_new(bg.prep_exec("SELECT * FROM questions WHERE lec = ?", vec![key.internal_unbox().clone()]));
    drop(bg);

    let make_questions = |questions_res: &Vec<Vec<Value>>, answers_res: &Vec<Vec<Value>>| {
        let mut answers = HashMap::new();
        for r in answers_res {
            let id: u64 = from_value(r[2].clone());
            let atext: String = from_value(r[3].clone());
            answers.insert(id, atext);
        }

        let mut questions: Vec<_> = questions_res
            .into_iter()
            .map(|r| {
                let id: u64 = from_value(r[1].clone());
                let answer = answers.get(&id).map(|s| s.to_owned());
                LectureQuestion {
                    id: id,
                    prompt: from_value(r[2].clone()),
                    answer: answer,
                }
            })
            .collect();
        questions.sort_by(|a, b| a.id.cmp(&b.id));
        questions
    };

    let questions = BBox::<Vec<LectureQuestion>>::sandbox_combine(questions_result, answers_result, make_questions);

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
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Redirect {
    let num = BBox::new(num);

    let mut bg = backend.lock().unwrap();
    let ts: Value = Local::now().naive_local().into();
    let grade: Value = 0.into();

    for (id, answer) in &data.answers {
        let rec: Vec<Value> = vec![
            apikey.user.into2::<Value>().internal_unbox().clone(),
            num.into2::<u64>().into2::<Value>().internal_unbox().clone(),
            (*id).into(),
            answer.into2::<Value>().internal_unbox().clone(),
            ts.clone(),
            grade.clone(),
        ];
        bg.replace("answers", rec);
    }

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
