use std::sync::{Arc, Mutex};

use mysql::prelude::FromValue;

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use serde::Serialize;

use mysql::from_value;

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;

use rocket::get;
use rocket::request::{FromRequest, Outcome, Request};
use rocket_dyn_templates::Template;

pub(crate) struct Manager;

#[derive(Debug)]
pub(crate) enum ManagerError {
    Unauthorized,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Manager {
    type Error = ManagerError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();

        let manager = if cfg.managers.contains(&apikey.user) {
            Some(Manager)
        } else {
            None
        };

        manager.into_outcome((Status::Unauthorized, ManagerError::Unauthorized))
    }
}

#[derive(Serialize)]
pub(crate) struct Aggregate<T: Serialize> {
    property: T,
    average: f64,
}

#[derive(Serialize)]
struct AggregateGenderContext {
    aggregate: Vec<Aggregate<String>>,
    parent: String,
}

#[derive(Serialize)]
struct AggregateRemoteContext {
    aggregate: Vec<Aggregate<bool>>,
    parent: String,
}

fn transform<T: Serialize + FromValue>(agg: Vec<Vec<mysql::Value>>) -> Vec<Aggregate<T>> {
    agg.into_iter()
        .map(|r| Aggregate {
            property: from_value(r[0].clone()),
            average: from_value(r[1].clone()),
        })
        .collect()
}

#[get("/gender")]
pub(crate) fn get_aggregate_gender(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec("SELECT * from agg_gender", vec![]);
    drop(bg);

    let ctx = AggregateGenderContext {
        aggregate: transform(grades),
        parent: String::from("layout"),
    };

    Template::render("manage/aggregate", &ctx)
}

#[get("/remote_buggy")]
pub(crate) fn get_aggregate_remote_buggy(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec("SELECT * from agg_remote", vec![]);
    drop(bg);

    let ctx = AggregateRemoteContext {
        aggregate: transform(grades),
        parent: String::from("layout"),
    };

    Template::render("manage/aggregate", &ctx)
}

#[get("/remote")]
pub(crate) fn get_aggregate_remote(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec("SELECT * from agg_remote WHERE ucount >= 10", vec![]);
    drop(bg);

    let ctx = AggregateRemoteContext {
        aggregate: transform(grades),
        parent: String::from("layout"),
    };

    Template::render("manage/aggregate", &ctx)
}

#[derive(Serialize, Clone)]
pub(crate) struct InfoForEmployers {
    email: String,
    average_grade: f64,
}

#[derive(Serialize)]
struct InfoForEmployersContext {
    users: Vec<InfoForEmployers>,
    parent: String,
}

#[get("/employers")]
pub(crate) fn get_list_for_employers(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    _config: &State<Config>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * from employers_release WHERE consent = 1", vec![]);
    drop(bg);

    let users = res
        .into_iter()
        .map(|r| InfoForEmployers {
            email: from_value(r[0].clone()),
            average_grade: from_value(r[1].clone()),
        })
        .collect();

    let ctx = InfoForEmployersContext {
        users,
        parent: "layout".into(),
    };
    Template::render("manage/users", &ctx)
}

#[get("/employers_buggy")]
pub(crate) fn get_list_for_employers_buggy(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    _config: &State<Config>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * from employers_release", vec![]);
    drop(bg);

    let users = res
        .into_iter()
        .map(|r| InfoForEmployers {
            email: from_value(r[0].clone()),
            average_grade: from_value(r[1].clone()),
        })
        .collect();

    let ctx = InfoForEmployersContext {
        users,
        parent: "layout".into(),
    };
    Template::render("manage/users", &ctx)
}
