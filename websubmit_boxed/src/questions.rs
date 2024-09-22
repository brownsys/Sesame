use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::naive::NaiveDateTime;
use chrono::Local;
use mysql::Value;
use rocket::State;

use crate::admin::Admin;
use alohomora::bbox::{BBox, BBoxRender};
use alohomora::context::Context;
use alohomora::db::{from_value, from_value_or_null};
use alohomora::fold::fold;
use alohomora::pcr::{PrivacyCriticalRegion, Signature};
use alohomora::policy::{AnyPolicy, NoPolicy};
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use alohomora::rocket::{get, post, BBoxForm, BBoxRedirect, BBoxTemplate, FromBBoxForm};
use alohomora::unbox::unbox;
use alohomora::AlohomoraType;

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;
use crate::helpers::{left_join, JoinIdx};
use crate::policies::{AnswerAccessPolicy, ContextData};

// TODO (allen): is this NoPolicy because it came from the user and we're going to write it (not for reading yet?)
#[derive(Debug, FromBBoxForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<u64, BBox<String, NoPolicy>>,
}

// TODO (allen): these are NoPolicy because not sensitive information? but answer could be right?
#[derive(BBoxRender, Clone)]
pub(crate) struct LectureQuestion {
    pub id: BBox<u64, NoPolicy>,
    pub prompt: BBox<String, NoPolicy>,
    pub answer: BBox<Option<String>, NoPolicy>,
}

// TODO (allen): do we need BBox's for context to our pages?
#[derive(BBoxRender)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: BBox<u8, NoPolicy>,
    pub questions: Vec<LectureQuestion>,
    pub parent: String,
}

#[derive(BBoxRender, Clone, AlohomoraType)]
#[alohomora_out_type(to_derive = [BBoxRender, Clone, Serialize])]
//#[derive(BBoxRender, Clone)]
pub struct LectureAnswer {
    pub id: BBox<u64, AnswerAccessPolicy>,
    pub user: BBox<String, AnswerAccessPolicy>,
    pub answer: BBox<String, AnswerAccessPolicy>,
    pub time: BBox<String, AnswerAccessPolicy>,
    pub grade: BBox<u64, AnswerAccessPolicy>,
}

// TODO (allen): do we need BBox's for context to our pages? and what kind of policy should they have?
#[derive(BBoxRender)]
pub struct LectureAnswersContext {
    pub lec_id: BBox<u8, NoPolicy>,
    pub answers: BBox<Vec<LectureAnswerOut>, AnswerAccessPolicy>,
    pub parent: String,
}

#[derive(BBoxRender)]
pub struct NaiveLectureAnswersContext {
    pub lec_id: BBox<u8, NoPolicy>,
    pub answers: Vec<LectureAnswer>,
    pub parent: String,
}

// TODO (allen): these are NoPolicy because not sensitive user information?
#[derive(BBoxRender, AlohomoraType)]
#[alohomora_out_type(to_derive = [BBoxRender, Clone])]
struct LectureListEntry {
    id: BBox<u64, NoPolicy>,
    label: BBox<String, NoPolicy>,
    num_qs: BBox<u64, NoPolicy>,
    num_answered: u64,
}

// TODO (allen): do we need BBox's for context to our pages? and what kind of policy should they have?
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
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
        (),
        context.clone(),
    );
    drop(bg);

    let admin: BBox<bool, NoPolicy> = apikey.user.into_ppr(PrivacyPureRegion::new(|email| {
        config.admins.contains(&email)
    }));

    let lecs: Vec<LectureListEntry> = res
        .into_iter()
        .map(|r: Vec<BBox<Value, AnyPolicy>>| LectureListEntry {
            id: from_value(r[0].clone()).unwrap(),
            label: from_value(r[1].clone()).unwrap(),
            num_qs: r[2]
                .clone()
                .specialize_policy()
                .unwrap()
                .into_ppr(PrivacyPureRegion::new(|v| match v {
                    Value::NULL => 0u64,
                    v => mysql::from_value(v),
                })),
            num_answered: 0u64,
        })
        .collect();

    let ctx = LectureListContext {
        admin,
        lectures: lecs,
        parent: "layout".into(),
    };

    BBoxTemplate::render("leclist", &ctx, context)
}

#[get("/naive/<num>")]
pub(crate) fn naive_answers(
    _admin: Admin,
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let key = num.clone().into_bbox::<u64, NoPolicy>();
    let res = bg.prep_exec(
        "SELECT * FROM answers WHERE lec = ?",
        (key,),
        context.clone(),
    );
    drop(bg);

    // Wraps incoming column data in LectureAnswer format
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

    let ctx = NaiveLectureAnswersContext {
        lec_id: num,
        answers,
        parent: "layout".into(),
    };
    BBoxTemplate::render("answers", &ctx, context)
}

#[get("/<num>")]
pub(crate) fn composed_answers(
    _admin: Admin,
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let key = num.clone().into_bbox::<u64, NoPolicy>();
    let res = bg.prep_exec(
        "SELECT * FROM answers WHERE lec = ?",
        (key,),
        context.clone(),
    );
    drop(bg);

    // Wraps incoming column data in LectureAnswer format
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

    let outer_box_answers = fold(answers)
        .unwrap()
        .specialize_policy::<AnswerAccessPolicy>()
        .unwrap();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: outer_box_answers,
        parent: "layout".into(),
    };
    BBoxTemplate::render("answers", &ctx, context)
}

#[get("/discussion_leaders/<num>")]
pub(crate) fn answers_for_discussion_leaders(
    num: BBox<u8, NoPolicy>,
    apikey: ApiKey,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let key = num.clone().into_bbox::<u64, NoPolicy>();

    let is_discussion_leader = {
        let mut bg = backend.lock().unwrap();
        let vec = bg.prep_exec(
            "SELECT * FROM discussion_leaders WHERE lec = ? AND email = ?",
            (key.clone(), apikey.user),
            context.clone(),
        );
        vec.len() > 0
    };

    if !is_discussion_leader {
        panic!()
    }

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM answers WHERE lec = ?",
        (key,),
        context.clone(),
    );
    drop(bg);

    // Wraps incoming column data in LectureAnswer format
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

    let outer_box_answers = fold(answers)
        .unwrap()
        .specialize_policy::<AnswerAccessPolicy>()
        .unwrap();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: outer_box_answers,
        parent: "layout".into(),
    };
    BBoxTemplate::render("answers", &ctx, context)
}

#[get("/discussion_leaders/naive/<num>")]
pub(crate) fn answers_for_discussion_leaders_naive(
    num: BBox<u8, NoPolicy>,
    apikey: ApiKey,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let key = num.clone().into_bbox::<u64, NoPolicy>();

    let is_discussion_leader = {
        let mut bg = backend.lock().unwrap();
        let vec = bg.prep_exec(
            "SELECT * FROM discussion_leaders WHERE lec = ? AND email = ?",
            (key.clone(), apikey.user),
            context.clone(),
        );
        vec.len() > 0
    };

    if !is_discussion_leader {
        panic!()
    }

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM answers WHERE lec = ?",
        (key,),
        context.clone(),
    );
    drop(bg);

    // Wraps incoming column data in LectureAnswer format
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

    let ctx = NaiveLectureAnswersContext {
        lec_id: num,
        answers,
        parent: "layout".into(),
    };
    BBoxTemplate::render("answers", &ctx, context)
}

#[get("/<num>")]
pub(crate) fn questions(
    apikey: ApiKey,
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let key = num.clone().into_bbox::<u64, NoPolicy>();

    let answers_result = bg.prep_exec(
        "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
        (key.clone(), apikey.user),
        context.clone(),
    );
    let questions_result = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ?",
        (key,),
        context.clone(),
    );
    drop(bg);

    let questions: BBox<Vec<Vec<Value>>, AnyPolicy> = execute_pure(
        (questions_result, answers_result),
        PrivacyPureRegion::new(|(questions, answers)| {
            let mut questions = left_join(
                questions,
                answers,
                1,
                2,
                vec![JoinIdx::Left(1), JoinIdx::Left(2), JoinIdx::Right(3)],
            );
            questions.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
            questions
        }),
    )
    .unwrap();

    let questions: Vec<BBox<Vec<Value>, AnyPolicy>> = questions.into();
    let questions = questions
        .into_iter()
        .map(|r: BBox<Vec<Value>, AnyPolicy>| {
            let r: Vec<BBox<Value, AnyPolicy>> = r.into();
            LectureQuestion {
                id: from_value(r[0].clone()).unwrap(),
                prompt: from_value(r[1].clone()).unwrap(),
                answer: from_value_or_null(r[2].clone()).unwrap(),
            }
        })
        .collect();
    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: questions,
        parent: "layout".into(),
    };

    BBoxTemplate::render("questions", &ctx, context)
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ContextData>,
) -> BBoxRedirect {
    let num = num.into_bbox::<u64, NoPolicy>();
    let ts: mysql::Value = Local::now().naive_local().into();
    let grade: mysql::Value = 0.into();

    let mut bg = backend.lock().unwrap();
    for (id, answer) in &data.answers {
        bg.replace(
            "answers",
            (
                apikey.user.clone(),
                num.clone(),
                *id,
                answer.clone(),
                ts.clone(),
                grade.clone(),
            ),
            context.clone(),
        );
    }

    if config.send_emails {
        let data = (data.answers.clone(), num, apikey.user);
        let result = unbox(
            data,
            context,
            PrivacyCriticalRegion::new(
                |(answers, num, user): (HashMap<u64, String>, u64, String), _| {
                    let answer_log = format!(
                        "{}",
                        answers
                            .iter()
                            .map(|(i, t)| format!("Question {}:\n{}", i, t))
                            .collect::<Vec<String>>()
                            .join("\n-----\n"),
                    );

                    // config is component of the context -> has passed policy check at this point
                    let recipients = if num < 90 {
                        config.staff.clone()
                    } else {
                        config.admins.clone()
                    };

                    email::send(
                        bg.log.clone(),
                        user,
                        recipients,
                        format!("{} meeting {} questions", config.class, num,),
                        answer_log,
                    )
                    .expect("failed to send email")
                },
                Signature{username: "corinnt", signature: "?"}, 
            Signature{username: "corinnt", signature: "?"},
            Signature{username: "corinnt", signature: "?"}, 
            ),
            (),
        );
        result.unwrap();
    }
    drop(bg);

    BBoxRedirect::to2("/leclist")
}
