use std::sync::{Arc, Mutex};

use mysql::prelude::FromValue;

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use serde::Serialize;

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::helpers::average;

use crate::policies::ContextData;
use bbox::policy::Context;
use bbox::db::{from_value};
use bbox::bbox::{BBox, sandbox_execute};
use bbox::rocket::{BBoxRequest, BBoxRequestOutcome, BBoxTemplate, FromBBoxRequest};
use bbox_derive::{BBoxRender, get};

pub(crate) struct Manager;

#[derive(Debug)]
pub(crate) enum ManagerError {
    Unauthorized,
}

#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for Manager {
    type BBoxError = ManagerError;

    async fn from_bbox_request(request: &'r BBoxRequest<'r, '_>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();
        let context = request
            .guard::<Context<ApiKey, ContextData>>()
            .await
            .unwrap();
        let manager = apikey
            .user
            .sandbox_execute(|user| cfg.managers.contains(user));

        // TODO(babman): find a better way here.
        let res = if *manager.unbox(&context) {
            Some(Manager)
        } else {
            None
        };

        res.into_outcome((Status::Unauthorized, ManagerError::Unauthorized))
    }
}

#[derive(BBoxRender)]
pub(crate) struct Aggregate<T: Serialize> {
    property: BBox<T>,
    average: BBox<f64>,
}

#[derive(BBoxRender)]
struct AggregateContext {
    aggregates_per_user: Vec<Aggregate<String>>,
    aggregates_per_gender: Vec<Aggregate<String>>,
    aggregates_per_remote: Vec<Aggregate<u8>>,
    parent: String,
}

fn transform<T: Serialize + FromValue>(agg: BBox<Vec<Vec<mysql::Value>>>) -> Vec<Aggregate<T>> {
    let agg: Vec<BBox<Vec<mysql::Value>>> = agg.into();
    agg.into_iter()
        .map(|r| {
            let r: Vec<BBox<mysql::Value>> = r.into();
            Aggregate {
                property: from_value(r[0].clone()),
                average: from_value(r[1].clone()),
            }
        })
        .collect()
}

#[get("/")]
pub(crate) fn get_aggregate_grades(
    _manager: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec(
        "SELECT pseudonym, gender, is_remote, grade FROM users JOIN answers ON users.email = answers.email;",
        vec![]);
    drop(bg);

    let user_agg = sandbox_execute(&grades, |grades| average::<String>(3, 0, grades));
    let gender_agg = sandbox_execute(&grades, |grades| average::<String>(3, 1, grades));
    let remote_agg = sandbox_execute(&grades, |grades| average::<bool>(3, 2, grades));

    let ctx = AggregateContext {
        aggregates_per_user: transform(user_agg),
        aggregates_per_gender: transform(gender_agg),
        aggregates_per_remote: transform(remote_agg),
        parent: String::from("layout"),
    };

    BBoxTemplate::render("manage/users", &ctx, &context)
}
