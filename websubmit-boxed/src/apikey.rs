use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rocket::form::Form;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use std::sync::{Arc, Mutex};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use bbox::{BBox, BBoxRender};
use bbox_derive::{BBoxRender};
use bbox::db::from_value;

// These should be imported automatically for us by derive..
use std::collections::BTreeMap;
use bbox::ValueOrBBox;

// Errors that we may encounter when authenticating an ApiKey.
#[derive(Debug)]
pub(crate) enum ApiKeyError {
    Ambiguous,
    Missing,
    BackendFailure,
}

// Check API key against database.
pub(crate) fn check_api_key(
    backend: &Arc<Mutex<MySqlBackend>>,
    key: &BBox<String>,
) -> Result<BBox<String>, ApiKeyError> {
    let mut bg = backend.lock().unwrap();
    let rs = bg.prep_exec("SELECT * FROM users WHERE apikey = ?", vec![key.into()]);
    drop(bg);

    if rs.len() < 1 {
        Err(ApiKeyError::Missing)
    } else if rs.len() > 1 {
        Err(ApiKeyError::Ambiguous)
    } else if rs.len() >= 1 {
        // user email
        let user = from_value::<String>(rs[0][0].clone());
        Ok(user)
    } else {
        Err(ApiKeyError::BackendFailure)
    }
}

/// (username, apikey)
pub(crate) struct ApiKey {
    pub user: BBox<String>,
    pub key: BBox<String>,
}

// Auto construct ApiKey from every request using cookies.
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
            // TODO(babman): cookie must be bbox.
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|key: String| Some(BBox::internal_new(key)))
            .and_then(|key: BBox<String>| match check_api_key(&be, &key) {
                Ok(user) => Some(ApiKey { user: user, key: key }),
                Err(_) => None,
            })
            .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeyRequest {
    email: BBox<String>,
    gender: BBox<String>,
    age: BBox<u32>,
    ethnicity: BBox<String>,
    is_remote: BBox<bool>,
    education: BBox<String>,
}

#[derive(BBoxRender)]
pub(crate) struct ApiKeyResponse {
    apikey_email: BBox<String>,
    parent: String,
}

// TODO(babman): too many sandboxes, what is reasonable here?
#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: Form<ApiKeyRequest>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Template {
    let pseudonym: String = thread_rng()
      .sample_iter(&Alphanumeric)
      .take(16)
      .map(char::from)
      .collect();

    // generate an API key from email address
    let hash = data.email.sandbox_execute(|email| {
      let mut hasher = Sha256::new();
      hasher.input_str(email);
      // add a secret to make API keys unforgeable without access to the server
      hasher.input_str(&config.secret);
      hasher.result_str()
    });

    // Check if request corresponds to admin or manager.
    // TODO(babman): pure sandbox.
    let is_admin = data.email.sandbox_execute(|email| config.admins.contains(email));
    let is_manager = data.email.sandbox_execute(|email| config.managers.contains(email));
    let is_admin: BBox<i8> = is_admin.m_into2();
    let is_manager: BBox<i8> = is_manager.m_into2();

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "users",
        vec![data.email.clone().into(),
             hash.clone().into(),
             is_admin.into(),
             is_manager.into(),
             pseudonym.into(),
             data.gender.clone().into(),
             data.age.clone().into(),
             data.ethnicity.clone().into(),
             data.is_remote.clone().into(),
             data.education.clone().into()],
    );

    if config.send_emails {
        // TODO(babman): some context that represents sending an email; unbox given that context
        email::send(
            bg.log.clone(),
            "no-reply@csci2390-submit.cs.brown.edu".into(),
            vec![data.email.unbox("email").clone()],
            format!("{} API key", config.class),
            format!("Your {} API key is: {}\n", config.class, hash.unbox("email")),
        )
        .expect("failed to send API key email");
    }
    drop(bg);

    // return to user
    let ctx = ApiKeyResponse {
      apikey_email: data.email.clone(),
      parent: "layout".into(),
    };
    bbox::render("apikey/generate", &ctx).unwrap()
}

#[derive(Debug, FromForm)]
pub(crate) struct ApiKeySubmit {
    key: BBox<String>,
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
            eprintln!("No such API key: {:?}", data.key);
        }
        Err(ApiKeyError::Ambiguous) => {
            eprintln!("Ambiguous API key: {:?}", data.key);
        }
        Ok(_) => (),
    }

    if res.is_err() {
        Redirect::to("/")
    } else {
        // TODO(babman): should be able to store bboxes in cookies.
        let cookie = Cookie::build("apikey", data.key.internal_unbox().clone()).path("/").finish();
        cookies.add(cookie);
        Redirect::to("/leclist")
    }
}
