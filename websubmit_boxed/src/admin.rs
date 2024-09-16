use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use alohomora::bbox::{BBox, BBoxRender, EitherBBox};
use alohomora::context::Context;
use alohomora::db::from_value;
use alohomora::policy::{AnyPolicy, NoPolicy};
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use alohomora::rocket::{
    get, post, BBoxForm, BBoxRedirect, BBoxRequest, BBoxRequestOutcome, BBoxTemplate, FromBBoxForm,
    FromBBoxRequest,
};

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
impl<'a, 'r> FromBBoxRequest<'a, 'r> for Admin {
    type BBoxError = AdminError;

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();

        let admin = apikey.user.ppr(PrivacyPureRegion::new(|user: &String| {
            if cfg.admins.contains(&user) {
                Some(Admin)
            } else {
                None
            }
        }));

        let admin = match admin.transpose() {
            None => None,
            Some(_) => Some(Admin),
        };
        admin.into_outcome((Status::Unauthorized, AdminError::Unauthorized))
    }
}

#[derive(BBoxRender)]
struct LecAddContext {
    parent: String,
}

#[get("/")]
pub(crate) fn lec_add(context: Context<ContextData>) -> BBoxTemplate {
    let ctx = LecAddContext {
        parent: "layout".into(),
    };
    BBoxTemplate::render("admin/lecadd", &ctx, context)
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct AdminLecAdd {
    lec_id: BBox<u8, NoPolicy>,
    lec_label: BBox<String, NoPolicy>,
}

#[post("/", data = "<data>")]
pub(crate) fn lec_add_submit(
    _adm: Admin,
    data: BBoxForm<AdminLecAdd>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxRedirect {
    let data = data.into_inner();

    let lec_id = data.lec_id.into_bbox::<u64, NoPolicy>();

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert("lectures", (lec_id, data.lec_label), context);
    drop(bg);

    BBoxRedirect::to2("/leclist")
}

#[get("/<num>")]
pub(crate) fn lec(
    _adm: Admin,
    num: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let key = num.clone().into_bbox::<u64, NoPolicy>();

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ? ORDER BY q",
        (key,),
        context.clone(),
    );
    drop(bg);

    let questions: Vec<LectureQuestion> = res
        .into_iter()
        .map(|r| {
            let id = from_value(r[1].clone()).unwrap();
            LectureQuestion {
                id: id,
                prompt: from_value(r[2].clone()).unwrap(),
                answer: BBox::new(None, NoPolicy {}), //TODO(corinn) check fix - this was previously BBox::new(None, vec![])
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

    BBoxTemplate::render("admin/lec", &ctx, context)
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
    context: Context<ContextData>,
) -> BBoxRedirect {
    let data = data.into_inner();

    let mut bg = backend.lock().unwrap();
    bg.insert(
        "questions",
        (
            num.clone().into_bbox::<u64, NoPolicy>(),
            data.q_id.into_bbox::<u64, NoPolicy>(),
            data.q_prompt,
        ),
        context.clone(),
    );
    drop(bg);

    BBoxRedirect::to("/admin/lec/{}", (&num,), context)
}

#[get("/<num>/<qnum>")]
pub(crate) fn editq(
    _adm: Admin,
    num: BBox<u8, NoPolicy>,
    qnum: BBox<u8, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ?",
        (num.clone().into_bbox::<u64, NoPolicy>(),),
        context.clone(),
    );
    drop(bg);

    let mut ctx: HashMap<&str, EitherBBox<String, NoPolicy>> = HashMap::new();
    for r in res {
        let q = from_value::<u8, AnyPolicy>(r[1].clone()).unwrap();

        let q_matches = execute_pure(
            (q, qnum.clone()),
            PrivacyPureRegion::new(|(q, qnum)| {
                if q == qnum {
                    Some(())
                } else {
                    None
                }
            }),
        )
        .unwrap();

        if q_matches.transpose().is_some() {
            ctx.insert("lec_qprompt", from_value(r[2].clone()).unwrap().into());
        }
    }
    ctx.insert(
        "lec_id",
        num.into_ppr(PrivacyPureRegion::new(|num| format!("{}", num)))
            .into(),
    );
    ctx.insert(
        "lec_qnum",
        qnum.into_ppr(PrivacyPureRegion::new(|qnum| format!("{}", qnum)))
            .into(),
    );
    ctx.insert("parent", String::from("layout").into());
    BBoxTemplate::render("admin/lecedit", &ctx, context)
}

#[post("/editq/<num>", data = "<data>")]
pub(crate) fn editq_submit(
    _adm: Admin,
    num: BBox<u8, NoPolicy>,
    data: BBoxForm<AddLectureQuestionForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxRedirect {
    let data = data.into_inner();
    let mut bg = backend.lock().unwrap();
    bg.prep_exec(
        "UPDATE questions SET question = ? WHERE lec = ? AND q = ?",
        (
            data.q_prompt,
            num.clone().into_bbox::<u64, NoPolicy>(),
            data.q_id,
        ),
        context.clone(),
    );
    drop(bg);

    BBoxRedirect::to("/admin/lec/{}", (&num,), context)
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
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT email, is_admin, apikey FROM users",
        (),
        context.clone(),
    );
    drop(bg);

    let users = res
        .into_iter()
        .map(|r| User {
            email: from_value(r[0].clone()).unwrap(),
            apikey: from_value(r[2].clone()).unwrap(),
            is_admin: from_value(r[0].clone())
                .unwrap()
                .into_ppr(PrivacyPureRegion::new(|id| config.admins.contains(&id))),
        })
        .collect();

    let ctx = UserContext {
        users: users,
        parent: "layout".into(),
    };
    BBoxTemplate::render("admin/users", &ctx, context)
}
