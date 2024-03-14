use std::sync::{Arc, Mutex};

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::State;

use alohomora::bbox::{BBox, BBoxRender};
use alohomora::db::from_value;

use alohomora::policy::{NoPolicy, AnyPolicy};
use alohomora::context::Context;
use alohomora::pcr::PrivacyCriticalRegion;

use alohomora::rocket::{BBoxCookie, BBoxCookieJar, BBoxForm, BBoxRedirect, BBoxRequest, BBoxRequestOutcome, BBoxTemplate, FromBBoxRequest, post, FromBBoxForm};
use alohomora::unbox::unbox;

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
    pub user: BBox<String, NoPolicy>,
    #[allow(dead_code)]
    pub key: BBox<String, NoPolicy>,
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct ApiKeyRequest {
    email: BBox<String, NoPolicy>,
    gender: BBox<String, NoPolicy>,
    age: BBox<u32, NoPolicy>,
    ethnicity: BBox<String, NoPolicy>,
    is_remote: Option<BBox<bool, NoPolicy>>,
    education: BBox<String, NoPolicy>,
}

#[derive(BBoxRender)]
pub(crate) struct ApiKeyResponse {
    apikey_email: BBox<String, NoPolicy>,
    parent: String,
}

#[derive(Debug, FromBBoxForm)]
pub(crate) struct ApiKeySubmit {
    key: BBox<String, NoPolicy>,
}

// Check API key against database.
pub(crate) fn check_api_key(
    backend: &Arc<Mutex<MySqlBackend>>,
    key: &BBox<String, NoPolicy>,
) -> Result<BBox<String, NoPolicy>, ApiKeyError> {
    let mut bg = backend.lock().unwrap();
    //let key_clone = key.specialize_policy::<NoPolicy>().unwrap().clone().into();
    let rs = bg.prep_exec("SELECT * FROM users WHERE apikey = ?", (key.clone(),));
    drop(bg);

    let rs = rs.into_iter().map(|row| {
        row.into_iter().map(|cell| {
            cell.specialize_policy::<NoPolicy>().unwrap()
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    if rs.len() < 1 {
        Err(ApiKeyError::Missing)
    } else if rs.len() > 1 {
        Err(ApiKeyError::Ambiguous)
    } else if rs.len() >= 1 {
        // user email
        // cast back w/ .any_policy() bc from_value implemented for AnyPolicy
        // unwrap Result<BBox<T, P>, String> of from_value, 
        let user = from_value::<String, AnyPolicy>(rs[0][0].clone().into_any_policy()).unwrap();
        let unwrapped_user = user.specialize_policy::<NoPolicy>().unwrap();
        // rewrap with Result<BBox<String, AnyPolicy>, ApiKeyError>
        Ok(unwrapped_user) 
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
        let be = request
            .guard::<&State<Arc<Mutex<MySqlBackend>>>>()
            .await
            .unwrap();
        request
            .cookies()
            .get("apikey")
            .and_then(|cookie: BBoxCookie<'_, NoPolicy>| Some(cookie.into()))
            .and_then(|key: BBox<String, NoPolicy>| match check_api_key(&be, &key) {
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
    let hash = alohomora::sandbox::execute_sandbox(data.email.clone(), |email| {
        let mut hasher = Sha256::new();
        hasher.input_str(&email);
        // add a secret to make API keys unforgeable without access to the server
        hasher.input_str(&config.secret);
        hasher.result_str()
    });

    // Check if request corresponds to admin or manager.
    // TODO(babman): pure sandbox.
    let is_admin = alohomora::sandbox::execute_sandbox(data.email.clone(), |email| config.admins.contains(&email));
    let is_manager = alohomora::sandbox::execute_sandbox(data.email.clone(), |email| config.managers.contains(&email));
    let is_admin: BBox<i8, AnyPolicy> = is_admin.into_bbox();
    let is_manager: BBox<i8, AnyPolicy> = is_manager.into_bbox();

    // insert into MySql if not exists
    let mut bg = backend.lock().unwrap();
    bg.insert(
        "users",
        (
            data.email.clone(),
            hash.clone(),
            is_admin,
            is_manager,
            pseudonym,
            data.gender.clone(),
            data.age.clone(),
            data.ethnicity.clone(),
            match &data.is_remote {
                Some(is_remote) => is_remote.clone(),
                None => BBox::new(false, NoPolicy {}),
            },
            data.education.clone(),
        ),
    );

    if config.send_emails {
        unbox(
            (data.email.clone(), hash),
            &context,
            PrivacyCriticalRegion::new(|(email, hash), _| {
                email::send(
                    bg.log.clone(),
                    "no-reply@csci2390-submit.cs.brown.edu".into(),
                    vec![email],
                    format!("{} API key", config.class),
                    format!(
                        "Your {} API key is: {}\n",
                        config.class,
                        hash
                    ),
                ).expect("failed to send API key email");
            }),
            ()).unwrap();
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
    cookies: BBoxCookieJar<'_, '_>,
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
        BBoxRedirect::to("/", ())
    } else {
        let cookie = BBoxCookie::build("apikey", data.key.clone())
            .path("/")
            .finish();
        cookies.add(cookie, &context).unwrap();
        BBoxRedirect::to("/leclist", ())
    }
}
