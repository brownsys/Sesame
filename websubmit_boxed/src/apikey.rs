use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::db::from_value;
use alohomora::pcr::{PrivacyCriticalRegion, Signature};
use alohomora::policy::{AnyPolicy, NoPolicy, Policy};
use alohomora::pure::PrivacyPureRegion;
use alohomora::AlohomoraType;

use alohomora::rocket::{
    post, BBoxCookie, BBoxCookieJar, BBoxForm, BBoxRedirect, BBoxRequest, BBoxRequestOutcome,
    FromBBoxForm, FromBBoxRequest, JsonResponse, OutputBBoxValue, ResponseBBoxJson,
};
use alohomora::sandbox::execute_sandbox;
use alohomora::unbox::unbox;

use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;
use crate::policies::{ContextData, QueryableOnly};

use websubmit_boxed_sandboxes::hash;

// Errors that we may encounter when authenticating an ApiKey.
#[derive(Debug)]
pub(crate) enum ApiKeyError {
    Ambiguous,
    Missing,
    BackendFailure,
}

/// (username, apikey)
#[derive(AlohomoraType, Clone)]
pub(crate) struct ApiKey {
    pub user: BBox<String, NoPolicy>,
    pub key: BBox<String, QueryableOnly>,
}

// Check API key against database.
pub(crate) fn check_api_key<P: Policy + Clone + 'static>(
    backend: &Arc<Mutex<MySqlBackend>>,
    key: &BBox<String, P>,
    context: Context<ContextData>,
) -> Result<BBox<String, NoPolicy>, ApiKeyError> {
    let mut bg = backend.lock().unwrap();
    let rs = bg.prep_exec(
        "SELECT * FROM users WHERE apikey = ?",
        (key.clone(),),
        context,
    );
    drop(bg);

    if rs.len() < 1 {
        Err(ApiKeyError::Missing)
    } else if rs.len() > 1 {
        Err(ApiKeyError::Ambiguous)
    } else if rs.len() == 1 {
        Ok(from_value(rs[0][0].clone()).unwrap())
    } else {
        Err(ApiKeyError::BackendFailure)
    }
}

// Auto construct ApiKey from every request using cookies.
#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ApiKey {
    type BBoxError = ApiKeyError;

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let context = request.guard().await.unwrap();
        let db: &State<Arc<Mutex<MySqlBackend>>> = request.guard().await.unwrap();

        request
            .cookies()
            .get::<QueryableOnly>("apikey")
            .and_then(|cookie: BBoxCookie<'_, QueryableOnly>| Some(cookie.into()))
            .and_then(
                |key: BBox<String, QueryableOnly>| match check_api_key(db, &key, context) {
                    Ok(user) => Some(ApiKey { user, key }),
                    Err(_) => None,
                },
            )
            .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

#[derive(FromBBoxForm)]
pub(crate) struct ApiKeyRequest {
    email: BBox<String, NoPolicy>,
    gender: BBox<String, NoPolicy>,
    age: BBox<u32, NoPolicy>,
    ethnicity: BBox<String, NoPolicy>,
    is_remote: Option<BBox<bool, NoPolicy>>,
    education: BBox<String, NoPolicy>,
    consent: Option<BBox<bool, NoPolicy>>,
}

pub(crate) struct ApiKeyResponse {
    email: BBox<String, AnyPolicy>,
    apikey: BBox<String, AnyPolicy>,
}

impl ResponseBBoxJson for ApiKeyResponse {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::Object(HashMap::from([
            (String::from("email"), self.email.to_json()),
            (String::from("apikey"), self.apikey.to_json()),
        ]))
    }
}

#[derive(FromBBoxForm)]
pub(crate) struct ApiKeySubmit {
    key: BBox<String, NoPolicy>,
}

#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: BBoxForm<ApiKeyRequest>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ContextData>,
) -> JsonResponse<ApiKeyResponse, ContextData> {
    let pseudonym: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // generate an API key from email address
    let hash = execute_sandbox::<hash, _, _>((data.email.clone(), config.secret.clone()));

    // Check if request corresponds to admin or manager.
    let is_manager = data.email.ppr(PrivacyPureRegion::new(|email| {
        config.managers.contains(email)
    }));
    let is_admin = data.email.ppr(PrivacyPureRegion::new(|email| {
        config.admins.contains(email)
    }));

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "users",
        (
            data.email.clone(),
            hash.clone(),
            is_admin.to_owned_policy(),
            is_manager.to_owned_policy(),
            pseudonym,
            data.gender.clone(),
            data.age.clone(),
            data.ethnicity.clone(),
            match &data.is_remote {
                Some(is_remote) => is_remote.clone(),
                None => BBox::new(false, NoPolicy {}),
            },
            data.education.clone(),
            match &data.consent {
                Some(consent) => consent.clone(),
                None => BBox::new(false, NoPolicy {}),
            },
        ),
        context.clone(),
    );

    if config.send_emails {
        unbox(
            (data.email.clone(), hash.clone()),
            context.clone(),
            PrivacyCriticalRegion::new(|(email, hash), _| {
                email::send(
                    bg.log.clone(),
                    "no-reply@csci2390-submit.cs.brown.edu".into(),
                    vec![email],
                    format!("{} API key", config.class),
                    format!("Your {} API key is: {}\n", config.class, hash),
                )
                .expect("failed to send API key email");
            }, 
            Signature{username: "corinnt", signature: "?"}, 
            Signature{username: "corinnt", signature: "?"},
            Signature{username: "corinnt", signature: "?"}, 
        ), 
        ())
        .unwrap();
    }
    drop(bg);

    // return to user
    let ctx = ApiKeyResponse {
        email: data.email.clone().into_any_policy(),
        apikey: hash.clone().into_any_policy(),
    };

    JsonResponse::from((ctx, context))
}

#[post("/", data = "<data>")]
pub(crate) fn check(
    data: BBoxForm<ApiKeySubmit>,
    cookies: BBoxCookieJar<'_, '_>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ContextData>,
) -> BBoxRedirect {
    // check that the API key exists and set cookie
    let res = check_api_key(&*backend, &data.key, context.clone());
    match res {
        Err(ApiKeyError::BackendFailure) => {
            eprintln!("Problem communicating with MySql backend");
        }
        Err(ApiKeyError::Missing) => {
            eprintln!("No such API key");
        }
        Err(ApiKeyError::Ambiguous) => {
            eprintln!("Ambiguous API key");
        }
        Ok(_) => (),
    }

    if res.is_err() {
        BBoxRedirect::to2("/")
    } else {
        let cookie = BBoxCookie::build("apikey", data.into_inner().key)
            .path("/")
            .finish();
        cookies.add(cookie, context).unwrap();
        BBoxRedirect::to2("/leclist")
    }
}
