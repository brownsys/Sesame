use std::sync::{Arc, Mutex};

use mysql::prelude::FromValue;

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use serde::Serialize;

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::policies::ContextData;

use alohomora::bbox::{BBox, BBoxRender};
use alohomora::context::Context;
use alohomora::db::from_value;
use alohomora::policy::AnyPolicy;
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{get, BBoxRequest, BBoxRequestOutcome, BBoxTemplate, FromBBoxRequest};

pub(crate) struct Manager;

#[derive(Debug)]
pub(crate) enum ManagerError {
    Unauthorized,
}

#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for Manager {
    type BBoxError = ManagerError;

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();

        let manager = apikey.user.ppr(PrivacyPureRegion::new(|user: &String| {
            if cfg.managers.contains(&user) {
                Some(Manager)
            } else {
                None
            }
        }));

        let manager = match manager.transpose() {
            None => None,
            Some(_) => Some(Manager),
        };
        manager.into_outcome((Status::Unauthorized, ManagerError::Unauthorized))
    }
}

#[derive(BBoxRender)]
pub(crate) struct Aggregate<T: Serialize> {
    property: BBox<T, AnyPolicy>,
    average: BBox<f64, AnyPolicy>,
}

#[derive(BBoxRender)]
struct AggregateGenderContext {
    aggregate: Vec<Aggregate<String>>,
    parent: String,
}

#[derive(BBoxRender)]
struct AggregateRemoteContext {
    aggregate: Vec<Aggregate<bool>>,
    parent: String,
}

fn transform<T: Serialize + FromValue>(
    agg: Vec<Vec<BBox<mysql::Value, AnyPolicy>>>,
) -> Vec<Aggregate<T>> {
    agg.into_iter()
        .map(|r| Aggregate {
            property: from_value(r[0].clone()).unwrap(),
            average: from_value(r[1].clone()).unwrap(),
        })
        .collect()
}

#[get("/gender")]
pub(crate) fn get_aggregate_gender(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec("SELECT * from agg_gender", (), context.clone());
    drop(bg);

    let ctx = AggregateGenderContext {
        aggregate: transform(grades),
        parent: String::from("layout"),
    };

    BBoxTemplate::render("manage/aggregate", &ctx, context)
}

#[get("/remote_buggy")]
pub(crate) fn get_aggregate_remote_buggy(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec("SELECT * from agg_remote", (), context.clone());
    drop(bg);

    let ctx = AggregateRemoteContext {
        aggregate: transform(grades),
        parent: String::from("layout"),
    };

    BBoxTemplate::render("manage/aggregate", &ctx, context)
}

#[get("/remote")]
pub(crate) fn get_aggregate_remote(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec(
        "SELECT * from agg_remote WHERE ucount >= 10",
        (),
        context.clone(),
    );
    drop(bg);

    let ctx = AggregateRemoteContext {
        aggregate: transform(grades),
        parent: String::from("layout"),
    };

    BBoxTemplate::render("manage/aggregate", &ctx, context)
}

#[derive(BBoxRender, Clone)]
pub(crate) struct InfoForEmployers {
    email: BBox<String, AnyPolicy>,
    average_grade: BBox<f64, AnyPolicy>,
}

#[derive(BBoxRender)]
struct InfoForEmployersContext {
    users: Vec<InfoForEmployers>,
    parent: String,
}

#[get("/employers")]
pub(crate) fn get_list_for_employers(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    _config: &State<Config>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT * from employers_release WHERE consent = 1",
        (),
        context.clone(),
    );
    drop(bg);

    let users = res
        .into_iter()
        .map(|r| InfoForEmployers {
            email: from_value(r[0].clone()).unwrap(),
            average_grade: from_value(r[1].clone()).unwrap(),
        })
        .collect();

    let ctx = InfoForEmployersContext {
        users: users,
        parent: "layout".into(),
    };
    BBoxTemplate::render("manage/users", &ctx, context)
}

#[get("/employers_buggy")]
pub(crate) fn get_list_for_employers_buggy(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    _config: &State<Config>,
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("SELECT * from employers_release", (), context.clone());
    drop(bg);

    let users = res
        .into_iter()
        .map(|r| InfoForEmployers {
            email: from_value(r[0].clone()).unwrap(),
            average_grade: from_value(r[1].clone()).unwrap(),
        })
        .collect();

    let ctx = InfoForEmployersContext {
        users: users,
        parent: "layout".into(),
    };
    BBoxTemplate::render("manage/users", &ctx, context)
}
