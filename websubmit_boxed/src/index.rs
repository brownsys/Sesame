use alohomora::context::Context;
use alohomora::policy::NoPolicy;
use alohomora::rocket::{get, BBoxCookieJar, BBoxRedirect};
use rocket::State;
use std::sync::{Arc, Mutex};

use crate::apikey;
use crate::backend::MySqlBackend;
use crate::policies;

#[get("/")]
pub(crate) fn index(
    cookies: BBoxCookieJar<'_, '_>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: Context<policies::ContextData>,
) -> BBoxRedirect {
    if let Some(cookie) = cookies.get::<NoPolicy>("apikey") {
        let apikey = cookie.into();
        match apikey::check_api_key(&*backend, &apikey, context) {
            Ok(_user) => BBoxRedirect::to2("/leclist"),
            Err(_) => BBoxRedirect::to2("/login"),
        }
    } else {
        BBoxRedirect::to2("/login")
    }
}
