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

use alohomora::context::Context;
use alohomora::db::from_value;
use alohomora::bbox::{BBox, BBoxRender};
use alohomora::policy::{AnyPolicy, NoPolicy};
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, BBoxTemplate, FromBBoxRequest, get};
use alohomora::pcr::PrivacyCriticalRegion;
use alohomora::pure::{execute_pure, PrivacyPureRegion};

pub(crate) struct Manager;

#[derive(Debug)]
pub(crate) enum ManagerError {
    Unauthorized,
}

#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for Manager {
    type BBoxError = ManagerError;

    async fn from_bbox_request(request: BBoxRequest<'a, 'r>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let apikey = request.guard::<ApiKey>().await.unwrap();
        let cfg = request.guard::<&State<Config>>().await.unwrap();

        let manager = apikey.user.ppr(PrivacyPureRegion::new(|user: &String|
            if cfg.managers.contains(&user) {
                Some(Manager)
            } else {
                None
            }
        ));

        manager
            .into_pcr(PrivacyCriticalRegion::new(|manager, _, _| manager), ())
            .into_outcome((Status::Unauthorized, ManagerError::Unauthorized))
    }
}

#[derive(BBoxRender)]
pub(crate) struct Aggregate<T: Serialize> {
    property: BBox<T, NoPolicy>,
    average: BBox<f64, NoPolicy>,
}

#[derive(BBoxRender)]
struct AggregateContext {
    aggregates_per_user: Vec<Aggregate<String>>,
    aggregates_per_gender: Vec<Aggregate<String>>,
    aggregates_per_remote: Vec<Aggregate<u8>>,
    parent: String,
}

fn transform<T: Serialize + FromValue>(agg: BBox<Vec<Vec<mysql::Value>>, AnyPolicy>) -> Vec<Aggregate<T>> {
    let agg: Vec<BBox<Vec<mysql::Value>, AnyPolicy>> = agg.into();
    agg.into_iter()
        .map(|r| {
            let r: Vec<BBox<mysql::Value, AnyPolicy>> = r.into();
            Aggregate {
                property: from_value(r[0].clone()).unwrap(),
                average: from_value(r[1].clone()).unwrap(),
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
        ());
    drop(bg);

    let user_agg = execute_pure(grades.clone(), PrivacyPureRegion::new(|grades| average::<String>(3, 0, grades))).unwrap();
    let gender_agg = execute_pure(grades.clone(), PrivacyPureRegion::new(|grades| average::<String>(3, 1, grades))).unwrap();
    let remote_agg = execute_pure(grades.clone(), PrivacyPureRegion::new(|grades| average::<bool>(3, 2, grades))).unwrap();

    let ctx = AggregateContext {
        aggregates_per_user: transform(user_agg),
        aggregates_per_gender: transform(gender_agg),
        aggregates_per_remote: transform(remote_agg),
        parent: String::from("layout"),
    };

    BBoxTemplate::render("manage/users", &ctx, &context)
}
