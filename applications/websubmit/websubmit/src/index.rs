use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, State};

use std::sync::{Arc, Mutex};

use crate::apikey;
use crate::backend::MySqlBackend;

#[get("/")]
pub(crate) fn index(
    cookies: &CookieJar<'_>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Redirect {
    if let Some(cookie) = cookies.get("apikey") {
        let apikey: String = cookie.value().parse().ok().unwrap();
        // TODO validate API key
        match apikey::check_api_key(&*backend, &apikey) {
            Ok(_user) => Redirect::to("/leclist"),
            Err(_) => Redirect::to("/login"),
        }
    } else {
        Redirect::to("/login")
    }
}
