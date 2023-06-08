use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use mysql::prelude::FromValue;

use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::State;
use rocket_dyn_templates::Template;

use serde::Serialize;

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::helpers::average;

use bbox::{BBox, BBoxRender};
use bbox::context::Context;
use bbox_derive::BBoxRender;
use bbox::db::{from_value, from_value_or_null};
use crate::policies::ContextData;

pub(crate) struct Manager;

#[derive(Debug)]
pub(crate) enum ManagerError {
    Unauthorized,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Manager {
    type Error = ManagerError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();
        let context = request.guard::<Context<ApiKey, ContextData>>().await.unwrap();
        let manager = apikey.user.sandbox_execute(|user| cfg.managers.contains(user));
        
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
    context: Context<ApiKey, ContextData>

) -> Template {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec(
        "SELECT pseudonym, gender, is_remote, grade FROM users JOIN answers ON users.email = answers.email;",
        vec![]);
    drop(bg);

    let user_agg = bbox::sandbox_execute(&grades, |grades| average::<String>(3, 0, grades));
    let gender_agg = bbox::sandbox_execute(&grades, |grades| average::<String>(3, 1, grades));
    let remote_agg = bbox::sandbox_execute(&grades, |grades| average::<bool>(3, 2, grades));

    let ctx = AggregateContext {
        aggregates_per_user: transform(user_agg),
        aggregates_per_gender: transform(gender_agg),
        aggregates_per_remote: transform(remote_agg),
        parent: String::from("layout"),
    };

    bbox::render("manage/users", &ctx, &context).unwrap()
}
