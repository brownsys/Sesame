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
    property: BBox<T, AnyPolicy>,
    average: BBox<f64, AnyPolicy>,
}

#[derive(BBoxRender)]
struct AggregateContext {
    aggregate: Vec<Aggregate<bool>>,
    parent: String,
}

fn transform<T: Serialize + FromValue>(agg: Vec<Vec<BBox<mysql::Value, AnyPolicy>>>) -> Vec<Aggregate<T>> {
    agg.into_iter()
        .map(|r| {
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
    context: Context<ContextData>,
) -> BBoxTemplate {
    let mut bg = backend.lock().unwrap();
    println!("THIS BEFORE QUERY");
    let grades = bg.prep_exec(
        "SELECT * from agg_remote",
        (),
        context.clone()
    );
    drop(bg);

    let ctx = AggregateContext {
        aggregate: transform(grades),
        parent: String::from("layout"),
    };

    BBoxTemplate::render("manage/users", &ctx, context)
}