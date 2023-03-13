use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use mysql::from_value;
use mysql::prelude::FromValue;
pub use mysql::Value;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::State;
use rocket_dyn_templates::Template;

use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use crate::config::Config;

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

        let res = if cfg.managers.contains(&apikey.user) {
            Some(Manager)
        } else {
            None
        };

        res.into_outcome((Status::Unauthorized, ManagerError::Unauthorized))
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct AnonymousAggregates<T> {
    property: T,
    average: f64,
}

#[derive(Serialize)]
struct AnonymousAggregatesContext {
    aggregates_per_user: Vec<AnonymousAggregates<String>>,
    aggregates_per_gender: Vec<AnonymousAggregates<String>>,
    aggregates_per_remote: Vec<AnonymousAggregates<u8>>,
    parent: &'static str,
}

fn get_aggregate<T: Eq + Hash + FromValue>(by_idx: usize, grade_idx: usize, data: &Vec<Vec<Value>>)
                                           -> Vec<AnonymousAggregates<T>> {
    data.iter()
        .fold(HashMap::new(), |mut m, r| {
            m.entry(from_value(r[by_idx].clone()))
                .and_modify(|v: &mut Vec<u64>| v.push(from_value(r[grade_idx].clone())))
                .or_insert(vec![from_value(r[grade_idx].clone())]);
            m
        }).into_iter()
        .map(|(pseudonym, grades)|
            AnonymousAggregates {
                property: pseudonym,
                average: grades.iter().sum::<u64>() as f64 / grades.len() as f64,
            }).collect()
}

#[get("/")]
pub(crate) fn get_aggregate_grades(
    _adm: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let grades = bg.prep_exec("SELECT pseudonym, gender, is_remote, grade FROM users JOIN answers ON users.email = answers.email;", vec![]);
    drop(bg);

    let aggregates_per_user = get_aggregate(0, 3, &grades);
    let aggregates_per_gender = get_aggregate(1, 3, &grades);
    let aggregates_per_remote = get_aggregate(2, 3, &grades);

    let ctx = AnonymousAggregatesContext {
        aggregates_per_user,
        aggregates_per_gender,
        aggregates_per_remote,
        parent: "layout",
    };
    Template::render("manage/users", &ctx)
}