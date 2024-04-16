use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;

use sha2::{Sha256, Digest};

use mysql::from_value;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use rocket::form::{Form, FromForm};
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::{post, State};
use rocket::serde::{Serialize, json::Json};

use std::sync::{Arc, Mutex};

/// (username, apikey)
#[allow(unused_tuple_struct_fields)]
pub(crate) struct ApiKey {
    pub user: String,
    #[allow(dead_code)]
    pub key: String,
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeyRequest {
    email: String,
    gender: String,
    age: u32,
    ethnicity: String,
    is_remote: bool,
    education: String,
    consent: bool,
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeySubmit {
    key: String,
}

#[derive(Debug)]
pub(crate) enum ApiKeyError {
    Ambiguous,
    Missing,
    BackendFailure,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct ApiKeyResponse {
    email: String,
    apikey: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let be = request
            .guard::<&State<Arc<Mutex<MySqlBackend>>>>()
            .await
            .unwrap();
        request
            .cookies()
            .get("apikey")
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|key: String| match check_api_key(&be, &key) {
                Ok(user) => Some(ApiKey { user, key }),
                Err(_) => None,
            })
            .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: Form<ApiKeyRequest>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Json<ApiKeyResponse> {
    let pseudonym: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // generate an API key from email address
    let mut hasher = Sha256::new();
    hasher.update(&data.email);
    hasher.update(&config.secret);
    let hash = format!("{:x}", hasher.finalize());

    let is_manager = if config.managers.contains(&data.email) {
        1.into()
    } else {
        0.into()
    };

    let is_admin = if config.admins.contains(&data.email) {
        1.into()
    } else {
        0.into()
    };

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "users",
        vec![
            data.email.as_str().into(), // do i need as_str
            hash.as_str().into(),
            is_admin,
            is_manager,
            pseudonym.into(),
            data.gender.as_str().into(),
            data.age.into(),
            data.ethnicity.as_str().into(),
            data.is_remote.into(),
            data.education.as_str().into(),
            data.consent.into(),
        ],
    );

    if config.send_emails {
        email::send(
            bg.log.clone(),
            "no-reply@csci2390-submit.cs.brown.edu".into(),
            vec![data.email.clone()],
            format!("{} API key", config.class),
            format!("Your {} API key is: {}\n", config.class, hash.as_str(),),
        )
        .expect("failed to send API key email");
    }
    drop(bg);

    // return to user
    let ctx = ApiKeyResponse {
        email: data.email.clone(),
        apikey: hash.clone(),
    };

    Json(ctx)
}

pub(crate) fn check_api_key(
    backend: &Arc<Mutex<MySqlBackend>>,
    key: &str,
) -> Result<String, ApiKeyError> {
    let mut bg = backend.lock().unwrap();
    let rs = bg.prep_exec("SELECT * FROM users WHERE apikey = ?", vec![key.into()]);
    drop(bg);

    if rs.len() < 1 {
        Err(ApiKeyError::Missing)
    } else if rs.len() > 1 {
        Err(ApiKeyError::Ambiguous)
    } else if rs.len() >= 1 {
        // user email
        Ok(from_value::<String>(rs[0][0].clone()))
    } else {
        Err(ApiKeyError::BackendFailure)
    }
}

#[post("/", data = "<data>")]
pub(crate) fn check(
    data: Form<ApiKeySubmit>,
    cookies: &CookieJar<'_>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Redirect {
    // check that the API key exists and set cookie
    let res = check_api_key(&*backend, &data.key);
    match res {
        Err(ApiKeyError::BackendFailure) => {
            eprintln!("Problem communicating with MySql backend");
        }
        Err(ApiKeyError::Missing) => {
            eprintln!("No such API key: {}", data.key);
        }
        Err(ApiKeyError::Ambiguous) => {
            eprintln!("Ambiguous API key: {}", data.key);
        }
        Ok(_) => (),
    }

    if res.is_err() {
        Redirect::to("/")
    } else {
        let cookie = Cookie::build("apikey", data.key.clone()).path("/").finish();
        cookies.add(cookie);
        Redirect::to("/leclist")
    }
}
