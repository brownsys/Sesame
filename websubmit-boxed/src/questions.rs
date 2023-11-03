use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash};
use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use chrono::Local;
use rocket::State;

use crate::admin::Admin;
use bbox::context::Context;
use bbox::db::{from_value, from_value_or_null};
use bbox::bbox::{BBox, sandbox_combine};
use bbox::rocket::{BBoxDataField, BBoxForm, BBoxFormResult, BBoxRedirect, BBoxTemplate, FromBBoxFormField};
use bbox_derive::{BBoxRender, FromBBoxForm, get, post};
use bbox::policy::{NoPolicy, AnyPolicy}; //{AnyPolicy, NoPolicy, PolicyAnd, SchemaPolicy};

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;
use crate::helpers::{left_join, JoinIdx};
use crate::policies::ContextData;

// TODO(babman): what about data that should not be bboxed!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fake64(u64);

#[rocket::async_trait]
impl<'r> FromBBoxFormField<'r> for Fake64 {
    async fn from_bbox_data<'i>(field: BBoxDataField<'r, 'i>) -> BBoxFormResult<'r, Self> {
        let boxed = BBox::<u64, NoPolicy>::from_bbox_data(field).await;
        match boxed {
            Ok(boxed) => Ok(Fake64(boxed.discard_box().clone())), //TODO need type conversion btwn BBoxes
            Err(e) => Err(e)
        }
    }
}
impl Display for Fake64 {
    // Required method
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl Into<bbox::db::Param> for Fake64 {
    fn into(self) -> bbox::db::Param {
        self.0.into()
    }
}
// END OF TODO.

#[derive(Debug, FromBBoxForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<Fake64, BBox<String, NoPolicy>>, //(corinn) answer.into() 
}

#[derive(BBoxRender, Clone)]
pub(crate) struct LectureQuestion {
    pub id: BBox<u64, NoPolicy>,
    pub prompt: BBox<String, NoPolicy>,
    pub answer: BBox<Option<String>, NoPolicy>,
}

#[derive(BBoxRender)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: BBox<u8, NoPolicy>,
    pub questions: Vec<LectureQuestion>,
    pub parent: String,
}

#[derive(BBoxRender)]
pub struct LectureAnswer {
    pub id: BBox<u64, NoPolicy>,
    pub user: BBox<String, NoPolicy>,
    pub answer: BBox<String, NoPolicy>,
    pub time: BBox<String, NoPolicy>,
    pub grade: BBox<u64, NoPolicy>,
}

#[derive(BBoxRender)]
pub struct LectureAnswersContext {
    pub lec_id: BBox<u8, NoPolicy>,
    pub answers: Vec<LectureAnswer>,
    pub parent: String,
}

#[derive(BBoxRender)]
struct LectureListEntry {
    id: BBox<u64, NoPolicy>,
    label: BBox<String, NoPolicy>,
    num_qs: BBox<u64, NoPolicy>,
    num_answered: u64,
}

#[derive(BBoxRender)]
struct LectureListContext {
    admin: BBox<bool, NoPolicy>,
    lectures: Vec<LectureListEntry>,
    parent: String,
}

#[get("/")]
pub(crate) fn leclist(
    apikey: ApiKey,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
        vec![],
    );
    drop(bg);

    let res = res.into_iter().map(|row| { //HERE
        row.into_iter().map(|cell| {
            cell.specialize_policy::<NoPolicy>().unwrap()
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    // TODO(babman): pure sandbox.
    let admin: BBox<bool, NoPolicy> = apikey
        .user
        .sandbox_execute(|email| config.admins.contains(email));

    let lecs: Vec<LectureListEntry> = res
        .into_iter()
        .map(|r| LectureListEntry {
            id: from_value(r[0].clone().any_policy()).unwrap(), 
            label: from_value(r[1].clone().any_policy()).unwrap(),

            // TODO(babman): also pure sandbox.
            num_qs: r[2].sandbox_execute(|v| {
                if *v == mysql::Value::NULL {
                    0u64
                } else {
                    mysql::from_value(v.clone())
                }
            }),
            num_answered: 0u64,
        })
        .collect();

    let ctx = LectureListContext {
        admin,
        lectures: lecs,
        parent: "layout".into(),
    };

    BBoxTemplate::render("leclist", &ctx, &context)
}

#[get("/<num>")]
pub(crate) fn answers(
    _admin: Admin,
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let key = num.clone().into_bbox::<u64>();
    let res = bg.prep_exec("SELECT * FROM answers WHERE lec = ?", vec![key.into()]);
    drop(bg);

    let answers: Vec<LectureAnswer> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone().any_policy()).unwrap(), 
            user: from_value(r[0].clone().any_policy()).unwrap(), 
            answer: from_value(r[3].clone().any_policy()).unwrap(), 
            time: from_value::<NaiveDateTime, AnyPolicy>(r[4].clone().any_policy()).unwrap()
                .specialize_policy::<NoPolicy>().unwrap() 
                .sandbox_execute(|v| v.format("%Y-%m-%d %H:%M:%S").to_string()),
            grade: from_value(r[5].clone().any_policy()).unwrap(), 
        })
        .collect();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers,
        parent: "layout".into(),
    };

    BBoxTemplate::render("answers", &ctx, &context)
}

#[get("/<num>")]
pub(crate) fn questions(
    apikey: ApiKey,
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let key = num.clone().into_bbox::<u64>();

    let answers_result = bg.prep_exec(
        "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
        vec![key.clone().into(), apikey.user.into()],
    );
    let questions_result = bg.prep_exec("SELECT * FROM questions WHERE lec = ?", vec![key.into()]);
    drop(bg);

    let questions = sandbox_combine(
        questions_result,
        answers_result,
        |questions_res: Vec<Vec<mysql::Value>>, answers_res: Vec<Vec<mysql::Value>>| {
            let mut questions = left_join(
                questions_res,
                answers_res,
                1,
                2,
                vec![JoinIdx::Left(1), JoinIdx::Left(2), JoinIdx::Right(3)],
            );
            questions.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
            questions
        },
    );

    let questions: Vec<BBox<Vec<mysql::Value>, NoPolicy>> = questions.into();
    let questions = questions
        .into_iter()
        .map(|r| {
            let r: Vec<BBox<mysql::Value, NoPolicy>> = r.into();
            LectureQuestion {
                id: from_value(r[0].clone().any_policy()).unwrap(),
                prompt: from_value(r[1].clone().any_policy()).unwrap(),
                answer: from_value_or_null(r[2].clone().any_policy()).unwrap(),
            }
        })
        .collect();
    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: questions,
        parent: "layout".into(),
    };

    BBoxTemplate::render("questions", &ctx, &context)
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ApiKey, ContextData>,
) -> BBoxRedirect {
    let num: BBox<u64, NoPolicy> = num.into_bbox();
    let ts: mysql::Value = Local::now().naive_local().into();
    let grade: mysql::Value = 0.into();

    let mut bg = backend.lock().unwrap();
    for (id, answer) in &data.answers {
        bg.replace(
            "answers",
            vec![
                apikey.user.clone().into(),
                num.clone().into(),
                (*id).into(),
                answer.clone().into(),
                ts.clone().into(),
                grade.clone().into(),
            ],
        );
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
            format!(
                "{} meeting {} questions",
                config.class,
                *num.unbox(&context)
            ),
            answer_log,
        )
        .expect("failed to send email");
    }
    drop(bg);

    BBoxRedirect::to("/leclist", vec![])
}
