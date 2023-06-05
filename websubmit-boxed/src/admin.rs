use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::form::Form;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;

use bbox::{BBox, VBox, BBoxRender};
use bbox_derive::BBoxRender;
use bbox::db::{from_value};

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::questions::{LectureQuestion, LectureQuestionsContext};

pub(crate) struct Admin;

#[derive(Debug)]
pub(crate) enum AdminError {
    Unauthorized,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = AdminError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();
        let admin = apikey.user.sandbox_execute(|user| cfg.admins.contains(user));
        
        // TODO(babman): find a better way here.
        let res = if *admin.unbox("admin request") {
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
pub(crate) fn lec_add() -> Template {
    let ctx = LecAddContext {
        parent: "layout".into(),
    };
    bbox::render("admin/lecadd", &ctx).unwrap()
}

#[derive(Debug, FromForm)]
pub(crate) struct AdminLecAdd {
    lec_id: BBox<u8>,
    lec_label: BBox<String>,
}

#[post("/", data = "<data>")]
pub(crate) fn lec_add_submit(
    data: Form<AdminLecAdd>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Redirect {
    let lec_id = data.lec_id.into2::<u64>();

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "lectures",
        vec![
            lec_id.into(),
            data.lec_label.clone().into(),
        ],
    );
    drop(bg);

    bbox::redirect("/leclist", vec![])
}

#[get("/<num>")]
pub(crate) fn lec(
    num: BBox<u8>, 
    backend: &State<Arc<Mutex<MySqlBackend>>>
) -> Template {
    let key = num.into2::<u64>();

    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ? ORDER BY q",
        vec![key.into()],
    );
    drop(bg);

    let mut questions: Vec<LectureQuestion> = res
        .into_iter()
        .map(|r| {
            let id: BBox<u64> = from_value(r[1].clone());
            LectureQuestion {
                id: id,
                prompt: from_value(r[2].clone()),
                answer: BBox::new(None),
            }
        })
        .collect();
    // questions.sort_by(|a, b| a.id.cmp(&b.id));

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: questions,
        parent: "layout".into(),
    };

    bbox::render("admin/lec", &ctx).unwrap()
}


#[derive(Debug, FromForm)]
pub(crate) struct AddLectureQuestionForm {
    q_id: BBox<u64>,
    q_prompt: BBox<String>,
}

#[post("/<num>", data = "<data>")]
pub(crate) fn addq(
    _adm: Admin,
    num: BBox<u8>,
    data: Form<AddLectureQuestionForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "questions",
        vec![
            num.into2::<u64>().into(),
            data.q_id.into2::<u64>().into(),
            data.q_prompt.clone().into(),
        ],
    );
    drop(bg);

    bbox::redirect("/admin/lec/{}", vec![&num])
}

#[get("/<num>/<qnum>")]
pub(crate) fn editq(
    _adm: Admin,
    num: BBox<u8>,
    qnum: BBox<u8>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * FROM questions WHERE lec = ?",
        vec![num.into2::<u64>().into()],
    );
    drop(bg);

    let mut ctx: HashMap<&str, VBox<String>> = HashMap::new();
    for r in res {
        // TODO(babman): how to handle this?
        let q = from_value::<u64>(r[1].clone());
        if q.unbox("check") == qnum.into2::<u64>().unbox("check") {
            ctx.insert("lec_qprompt", from_value(r[2].clone()).into());
        }
    }
    ctx.insert("lec_id", num.format().into());
    ctx.insert("lec_qnum", qnum.format().into());
    ctx.insert("parent", String::from("layout").into());
    bbox::render("admin/lecedit", &ctx).unwrap()
}


#[post("/editq/<num>", data = "<data>")]
pub(crate) fn editq_submit(
    _adm: Admin,
    num: BBox<u8>,
    data: Form<AddLectureQuestionForm>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    bg.prep_exec(
        "UPDATE questions SET question = ? WHERE lec = ? AND q = ?",
        vec![
            data.q_prompt.clone().into(),
            num.into2::<u64>().into(),
            data.q_id.into2::<u64>().into(),
        ],
    );
    drop(bg);

    bbox::redirect("/admin/lec/{}", vec![&num])
}


#[derive(BBoxRender, Clone)]
pub(crate) struct User {
    email: BBox<String>,
    apikey: BBox<String>,
    is_admin: BBox<bool>,
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
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT email, is_admin, apikey FROM users", vec![]);
    drop(bg);

    let users: Vec<_> = res
        .into_iter()
        .map(|r| {
          let id = from_value::<String>(r[0].clone());
          User {
            email: from_value(r[0].clone()),
            apikey: from_value(r[2].clone()),
            is_admin: id.sandbox_execute(|v| config.admins.contains(v)),
          }
        })
        .collect();

    let ctx = UserContext {
        users: users,
        parent: "layout".into(),
    };
    bbox::render("admin/users", &ctx).unwrap()
}
