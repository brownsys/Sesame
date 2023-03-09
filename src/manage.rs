use std::sync::{Arc, Mutex};

use mysql::from_value;
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
pub(crate) struct AnonymousAggregate {
    pseudonym: String,
    average_grade: f64,
}

#[derive(Serialize)]
struct AnonymousAggregateContext {
    users: Vec<AnonymousAggregate>,
    parent: &'static str,
}

#[get("/")]
pub(crate) fn get_aggregate_grades(
    _adm: Manager,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec("select users.pseudonym, avg(answers.grade) from users join answers on users.email = answers.email group by users.pseudonym;", vec![]);
    drop(bg);

    let anonymous_aggregates: Vec<_> = res.into_iter().map(|r| AnonymousAggregate {
        pseudonym: from_value(r[0].clone()),
        average_grade: from_value(r[1].clone()),
    }).collect();

    let ctx = AnonymousAggregateContext {
        users: anonymous_aggregates,
        parent: "layout",
    };
    Template::render("manage/users", &ctx)
}