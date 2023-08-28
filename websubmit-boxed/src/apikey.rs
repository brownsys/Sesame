use std::sync::{Arc, Mutex};

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use bbox::bbox::BBox;
use bbox::db::from_value;
use bbox::policy::Context;
use bbox::rocket::{
    BBoxCookie, BBoxCookieJar, BBoxForm, BBoxRedirect, BBoxRequest, BBoxRequestOutcome,
    BBoxTemplate, FromBBoxRequest,
};

use bbox_derive::{post, BBoxRender, FromBBoxForm};

use crate::backend::MySqlBackend;
use crate::config::Config;
use crate::email;
use crate::policies::ContextData;

// Errors that we may encounter when authenticating an ApiKey.
#[derive(Debug)]
pub(crate) enum ApiKeyError {
    Ambiguous,
    Missing,
    BackendFailure,
}

/// (username, apikey)
#[allow(unused_tuple_struct_fields)]
pub(crate) struct ApiKey {
    pub user: BBox<String>,
    #[allow(dead_code)]
    pub key: BBox<String>,
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct ApiKeyRequest {
    email: BBox<String>,
    gender: BBox<String>,
    age: BBox<u32>,
    ethnicity: BBox<String>,
    is_remote: Option<BBox<bool>>,
    education: BBox<String>,
}

#[derive(BBoxRender)]
pub(crate) struct ApiKeyResponse {
    apikey_email: BBox<String>,
    parent: String,
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct ApiKeySubmit {
    key: BBox<String>,
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
// Auto construct ApiKey from every request using cookies.
#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for ApiKey {
    type BBoxError = ApiKeyError;

    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let be = request
            .guard::<&State<Arc<Mutex<MySqlBackend>>>>()
            .await
            .unwrap();
        request
            .cookies()
            .get("apikey")
            .and_then(|cookie| Some(cookie.value().into_bbox()))
            .and_then(|key: BBox<String>| match check_api_key(&be, &key) {
                Ok(user) => Some(ApiKey {
                    user: user,
                    key: key,
                }),
                Err(_) => None,
            })
            .into_outcome((Status::Unauthorized, ApiKeyError::Missing))
    }
}

// TODO(babman): too many sandboxes, what is reasonable here?
#[post("/", data = "<data>")]
pub(crate) fn generate(
    data: BBoxForm<ApiKeyRequest>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
    context: Context<ApiKey, ContextData>,
) -> BBoxTemplate {
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
    let is_admin = data
        .email
        .sandbox_execute(|email| config.admins.contains(email));
    let is_manager = data
        .email
        .sandbox_execute(|email| config.managers.contains(email));
    let is_admin: BBox<i8> = is_admin.into_bbox();
    let is_manager: BBox<i8> = is_manager.into_bbox();

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "users",
        vec![
            data.email.clone().into(),
            hash.clone().into(),
            is_admin.into(),
            is_manager.into(),
            pseudonym.into(),
            data.gender.clone().into(),
            data.age.clone().into(),
            data.ethnicity.clone().into(),
            match &data.is_remote {
                Some(is_remote) => is_remote.clone().into(),
                None => false.into(),
            },
            data.education.clone().into(),
        ],
    );

    if config.send_emails {
        // TODO(babman): some context that represents sending an email; unbox given that context
        email::send(
            bg.log.clone(),
            "no-reply@csci2390-submit.cs.brown.edu".into(),
            vec![data.email.unbox(&context).clone()],
            format!("{} API key", config.class),
            format!(
                "Your {} API key is: {}\n",
                config.class,
                hash.unbox(&context)
            ),
        )
        .expect("failed to send API key email");
    }
    drop(bg);

    // return to user
    let ctx = ApiKeyResponse {
        apikey_email: data.email.clone(),
        parent: "layout".into(),
    };

    BBoxTemplate::render("apikey/generate", &ctx, &context)
}

#[post("/", data = "<data>")]
pub(crate) fn check(
    data: BBoxForm<ApiKeySubmit>,
    cookies: &BBoxCookieJar<'_>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<ApiKey, ContextData>,
) -> BBoxRedirect {
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
        BBoxRedirect::to("/", vec![])
    } else {
        let cookie = BBoxCookie::build("apikey", data.key.clone(), &context)
            .path("/")
            .finish();
        cookies.add(cookie);
        BBoxRedirect::to("/leclist", vec![])
    }
}
