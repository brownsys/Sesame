use crate::admin::Admin;
use crate::apikey::{ApiKey, BBoxApiKey};
use crate::backend::{MySqlBackend, Value};
use crate::config::Config;
use crate::email;
use chrono::naive::NaiveDateTime;
use chrono::Local;
use mysql::from_value;
use rocket::form::{DataField, Form, FromForm, Options, ValueField};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use ndarray::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::bbox::BBox;

//pub(crate) enum LectureQuestionFormError {
//   Invalid,
//}

#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<u64, String>,
}

pub(crate) struct BoxedLectureQuestionSubmission {
    answers: HashMap<u64, BBox<String>>,
}

impl BoxedLectureQuestionSubmission {
    pub fn new(lqs: &HashMap<u64, String>) -> Self {
        let mut _self: Self = Self { answers: HashMap::new() };
        for answer in lqs.iter() {
            _self.answers.insert(*answer.0, BBox::new(answer.1.clone()));
        }
        _self
    }
}

#[derive(Debug, FromForm)]
pub(crate) struct EditGradeForm {
    grade: u64,
}

#[derive(Debug, FromForm)]
pub(crate) struct PredictGradeForm {
    time: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: u8,
    pub questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

#[derive(Serialize)]
struct LectureAnswer {
    id: u64,
    user: String,
    answer: String,
    time: String,
    grade: u64,
}

#[derive(Serialize)]
struct LectureAnswersContext {
    lec_id: u8,
    answers: Vec<LectureAnswer>,
    parent: &'static str,
}

#[derive(Serialize)]
struct LectureListEntry {
    id: u64,
    label: String,
    num_qs: u64,
    num_answered: u64,
}

#[derive(Serialize)]
struct LectureListContext {
    admin: bool,
    lectures: Vec<LectureListEntry>,
    parent: &'static str,
}

#[derive(Serialize)]
struct PredictContext {
    lec_id: u8,
    parent: &'static str,
}

#[derive(Serialize)]
struct PredictGradeContext {
    lec_id: u8,
    time: String,
    grade: f64,
    parent: &'static str,
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

    let user = apikey.user.clone();
    let admin = config.admins.contains(&user);

    let lecs: Vec<_> = res
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

    let ctx = LectureListContext {
        admin: admin,
        lectures: lecs,
        parent: "layout",
    };

    Template::render("leclist", &ctx)
}

#[get("/<num>")]
pub(crate) fn predict(
    _admin: Admin,
    num: u8,
    _backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let ctx = PredictContext {
        lec_id: num,
        parent: "layout",
    };
    Template::render("predict", &ctx)
}

#[post("/predict_grade/<num>", data = "<data>")]
pub(crate) fn predict_grade(
    _adm: Admin,
    num: u8,
    data: Form<PredictGradeForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let time = NaiveDateTime::parse_from_str(data.time.as_str(), "%Y-%m-%d %H:%M:%S");
    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();
    let res = bg.prep_exec("SELECT submitted_at, grade FROM answers WHERE lec = ?", vec![key]);
    drop(bg);
    let grades: Vec<[f64; 2]> = res
        .into_iter()
        .map(|r| [
            from_value::<NaiveDateTime>(r[0].clone()).timestamp() as f64,
            from_value::<u64>(r[1].clone()) as f64
        ])
        .collect();

    let array: Array2<f64> = Array2::from(grades);

    let (x, y) = (
        array.slice(s![.., 0..1]).to_owned(),
        array.column(1).to_owned()
    );

    let dataset = Dataset::new(x, y).with_feature_names(vec!["x", "y"]);

    let model_path = Path::new("model.json");

    let model = if model_path.exists() {
        println!("Loading the model from a file...");
        let mut file = File::open(model_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_value((&contents).parse().unwrap()).unwrap()
    } else {
        println!("Re-training the model and saving it to disk...");
        let lin_reg = LinearRegression::new();
        let model = lin_reg.fit(&dataset).unwrap();
        let serialized_model = serde_json::to_string(&model).unwrap();
        let mut file = File::create(model_path).unwrap();
        file.write_all(serialized_model.as_ref()).unwrap();
        model
    };

    let grade = model.params()[0] * (time.unwrap().timestamp() as f64) + model.intercept();

    let ctx = PredictGradeContext {
        lec_id: num,
        time: data.time.clone(),
        grade: grade,
        parent: "layout",
    };
    Template::render("predictgrade", &ctx)
}

#[get("/<num>")]
pub(crate) fn grades(
    _admin: Admin,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();
    let res = bg.prep_exec("SELECT * FROM answers WHERE lec = ?", vec![key]);
    drop(bg);
    let answers: Vec<_> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()),
            user: from_value(r[0].clone()),
            answer: from_value(r[3].clone()),
            time: from_value::<NaiveDateTime>(r[4].clone()).format("%Y-%m-%d %H:%M:%S").to_string(),
            grade: from_value(r[5].clone()),
        })
        .collect();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers,
        parent: "layout",
    };
    Template::render("grades", &ctx)
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
        "SELECT * FROM answers WHERE lec = ?",
        vec![(num as u64).into()],
    );
    drop(bg);

    let mut ctx = HashMap::new();
    for r in res {
        if from_value::<String>(r[0].clone()) == user && from_value::<u8>(r[2].clone()) == qnum {
            ctx.insert("answer", format!("{}", from_value::<String>(r[3].clone())));
            ctx.insert("grade", format!("{}", from_value::<u64>(r[5].clone())));
        }
    }
    ctx.insert("user", format!("{}", user));
    ctx.insert("lec_id", format!("{}", num));
    ctx.insert("lec_qnum", format!("{}", qnum));
    ctx.insert("parent", String::from("layout"));
    Template::render("gradeedit", &ctx)
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
        vec![
            (data.grade as u64).into(),
            user.into(),
            (num as u64).into(),
            (qnum as u64).into(),
        ],
    );
    drop(bg);

    Redirect::to(format!("/grades/{}", num))
}

#[get("/<num>")]
pub(crate) fn answers(
    _admin: Admin,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();
    let res = bg.prep_exec("SELECT * FROM answers WHERE lec = ?", vec![key]);
    drop(bg);
    let answers: Vec<_> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()),
            user: from_value(r[0].clone()),
            answer: from_value(r[3].clone()),
            time: from_value::<NaiveDateTime>(r[4].clone()).format("%Y-%m-%d %H:%M:%S").to_string(),
            grade: from_value(r[5].clone()),
        })
        .collect();

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers,
        parent: "layout",
    };
    Template::render("answers", &ctx)
}

#[get("/<num>")]
pub(crate) fn questions(
    apikey: ApiKey,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    use std::collections::HashMap;

    let mut bg = backend.lock().unwrap();
    let num = BBox::new(num);
    let key: BBox<Value> = num.into2::<u64>().into2();
    let apikey = BBoxApiKey::new(&apikey);

    let answers_res = BBox::new(bg.prep_exec(
        "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
        vec![key.internal_unbox().clone(), apikey.user.internal_unbox().clone().into()],
    ));

    let make_hashmap = |answers_res: &Vec<Vec<Value>>| {
        let mut answers = HashMap::new();
        for r in answers_res {
            let id: u64 = from_value(r[2].clone());
            let atext: String = from_value(r[3].clone());
            answers.insert(id, atext);
        }
        answers
    };

    let answers = answers_res.sandbox_execute(make_hashmap);

    let res = BBox::new(bg.prep_exec("SELECT * FROM questions WHERE lec = ?", vec![key.internal_unbox().clone()]));
    drop(bg);

    let make_questions = |res: &Vec<Vec<Value>>, answers: &HashMap<u64, String>| {
        let mut qs: Vec<_> = res
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
        qs.sort_by(|a, b| a.id.cmp(&b.id));
        qs
    };

    let qs = BBox::<Vec<LectureQuestion>>::sandbox_combine(res, answers, make_questions);

    let ctx = LectureQuestionsContext {
        lec_id: *num.internal_unbox(),
        questions: qs.internal_unbox().clone(),
        parent: "layout",
    };
    Template::render("questions", &ctx)
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Redirect {
    let apikey = BBoxApiKey::new(&apikey);
    let num = BBox::new(num);
    let data = BoxedLectureQuestionSubmission::new(&data.answers);

    let mut bg = backend.lock().unwrap();
    // let vnum: Value = (num as u64).into();
    let ts: Value = Local::now().naive_local().into();
    let grade: Value = 0.into();

    for (id, answer) in &data.answers {
        let rec: Vec<Value> = vec![
            apikey.user.internal_unbox().clone().into(),
            (num.internal_unbox().clone() as u64).into(),
            (*id).into(),
            answer.internal_unbox().clone().into(),
            ts.clone(),
            grade.clone(),
        ];
        bg.replace("answers", rec);
    }

    // TODO: some context that represents sending an email; unbox given that context
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
            format!("{} meeting {} questions", config.class, num.unbox("email")),
            answer_log,
        )
            .expect("failed to send email");
    }
    drop(bg);

    Redirect::to("/leclist")
}
