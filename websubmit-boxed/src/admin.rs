use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use bbox::context::Context;
use bbox::db::from_value;
use bbox::bbox::{BBox, EitherBBox};
use bbox::rocket::{BBoxForm, BBoxRedirect, BBoxRequest, BBoxRequestOutcome, BBoxTemplate, FromBBoxRequest};
use bbox_derive::{BBoxRender, FromBBoxForm, get, post};
use bbox::policy::{NoPolicy, AnyPolicy}; //{AnyPolicy, NoPolicy, PolicyAnd, SchemaPolicy};


use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::policies::ContextData;
use crate::questions::{LectureQuestion, LectureQuestionsContext};

pub(crate) struct Admin;

#[derive(Debug)]
pub(crate) enum AdminError {
    Unauthorized,
}

#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for Admin {
    type BBoxError = AdminError;

    async fn from_bbox_request(request: &'r BBoxRequest<'r, '_>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();
        let context = request
            .guard::<Context<ApiKey, ContextData>>()
            .await
            .unwrap();

        // TODO(babman): find a better way here.
        let admin = apikey
            .user
            .sandbox_execute(|user| cfg.admins.contains(user));
        let res = if *admin.unbox(&context) {
            Some(Admin)
        } else {
            None
        };

        res.into_outcome((Status::Unauthorized, AdminError::Unauthorized))
    }
}

#[derive(BBoxRender)]
struct LecAddContext {
    parent: String,
}

#[get("/")]
pub(crate) fn lec_add(context: Context<ApiKey, ContextData>) -> BBoxTemplate {
    let ctx = LecAddContext {
        parent: "layout".into(),
    };
    BBoxTemplate::render("admin/lecadd", &ctx, &context)
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct AdminLecAdd {
    lec_id: BBox<u8, NoPolicy>,
    lec_label: BBox<String, NoPolicy>,
}

#[post("/", data = "<data>")]
pub(crate) fn lec_add_submit(
    data: BBoxForm<AdminLecAdd>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> BBoxRedirect {
    let data = data.into_inner();

    let lec_id = data.lec_id.into_bbox::<u64>();

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "lectures",
        vec![lec_id.into(), data.lec_label.into()],
    );
    drop(bg);

    BBoxRedirect::to("/leclist", vec![])
}

#[get("/<num>")]
pub(crate) fn lec(
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let key = num.clone().into_bbox::<u64>();

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ? ORDER BY q",
        vec![key.into()],
    );
    drop(bg);

    let questions: Vec<LectureQuestion> = res
        .into_iter()
        .map(|r| {
            let id: BBox<u64, NoPolicy> = from_value(r[1].clone()).unwrap();
            LectureQuestion {
                id: id,
                prompt: from_value(r[2].clone()).unwrap(),
                answer: BBox::new(None, NoPolicy{}), //TODO(corinn) check fix - this was previously BBox::new(None, vec![])
            }
        })
        .collect();
    // TODO(babman): sorting.
    // questions.sort_by(|a, b| a.id.cmp(&b.id));

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: questions,
        parent: "layout".into(),
    };

    BBoxTemplate::render("admin/lec", &ctx, &context)
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct AddLectureQuestionForm {
    q_id: BBox<u64, NoPolicy>,
    q_prompt: BBox<String, NoPolicy>,
}

#[post("/<num>", data = "<data>")]
pub(crate) fn addq(
    _adm: Admin,
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<AddLectureQuestionForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> BBoxRedirect {
    let data = data.into_inner();

    let mut bg = backend.lock().unwrap();
    bg.insert(
        "questions",
        vec![
            num.clone().into_bbox::<u64>().into(),
            data.q_id.into_bbox::<u64>().into(),
            data.q_prompt.into(),
        ],
    );
    drop(bg);

    BBoxRedirect::to("/admin/lec/{}", vec![&num])
}

#[get("/<num>/<qnum>")]
pub(crate) fn editq(
    _adm: Admin,
    num: BBox<u8, NoPolicy>,
    qnum: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ?",
        vec![num.clone().into_bbox::<u64>().into()],
    );
    drop(bg);

    let mut ctx: HashMap<&str, EitherBBox<String, NoPolicy>> = HashMap::new();
    for r in res {
        // TODO(babman): how to handle this?
        let q = from_value::<u64, AnyPolicy>(r[1].clone().any_policy()).unwrap();
        if q.unbox(&context) == qnum.clone().into_bbox::<u64>().unbox(&context) {
            ctx.insert("lec_qprompt", from_value(r[2].clone()).unwrap().into());
        }
    }
    ctx.insert("lec_id", num.format().into());
    ctx.insert("lec_qnum", qnum.format().into());
    ctx.insert("parent", String::from("layout").into());
    BBoxTemplate::render("admin/lecedit", &ctx, &context)
}

#[post("/editq/<num>", data = "<data>")]
pub(crate) fn editq_submit(
    _adm: Admin,
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<AddLectureQuestionForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> BBoxRedirect {
    let data = data.into_inner();
    let mut bg = backend.lock().unwrap();
    bg.prep_exec(
        "UPDATE questions SET question = ? WHERE lec = ? AND q = ?",
        vec![
            data.q_prompt.clone().into(),
            num.clone().into_bbox::<u64>().into(),
            data.q_id.into_bbox::<u64>().into(),
        ],
    );
    drop(bg);

    BBoxRedirect::to("/admin/lec/{}", vec![&num])
}

#[derive(BBoxRender, Clone)]
pub(crate) struct User {
    email: BBox<String, NoPolicy>,
    apikey: BBox<String, NoPolicy>,
    is_admin: BBox<bool, NoPolicy>,
}

#[derive(BBoxRender)]
struct UserContext {
    users: Vec<User>,
    parent: String,
}

#[get("/")]
pub(crate) fn get_registered_users(
    _adm: Admin,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT email, is_admin, apikey FROM users", vec![]);
    drop(bg);

    let users: Vec<_> = res
        .into_iter()
        .map(|r| {
            let id = from_value::<String, AnyPolicy>(r[0].clone().any_policy()).unwrap()
                                                        .specialize_policy::<NoPolicy>().unwrap();  
            User {
                email: from_value(r[0].clone().any_policy()).unwrap(),
                apikey: from_value(r[2].clone().any_policy()).unwrap(),
                is_admin: id.sandbox_execute(|v| config.admins.contains(v)),
            }
        })
        .collect();

    let ctx = UserContext {
        users: users,
        parent: "layout".into(),
    };
    BBoxTemplate::render("admin/users", &ctx, &context)
}