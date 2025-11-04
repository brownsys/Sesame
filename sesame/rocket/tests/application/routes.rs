use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::State;
use sesame::pcon::PCon;
use sesame_rocket::rocket::{
    FromPConData, PConCookie, PConData, PConForm, PConRequest, PConResponseOutcome, PConTemplate,
};

use crate::application::context::AppContext;
use crate::application::db::DB;
use crate::application::policy::{AuthenticationCookiePolicy, WritePolicy};

// Logins in as a user.
pub async fn login<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    let context: AppContext = request.guard().await.unwrap();

    let username: PCon<String, AuthenticationCookiePolicy> = request.param(1).unwrap().unwrap();
    request
        .cookies()
        .add(PConCookie::new("user", username), context)
        .unwrap();

    PConResponseOutcome::from(request, "success")
}

// Post a grade: have to be admin.
pub async fn post_grade<'a, 'r>(
    request: PConRequest<'a, 'r>,
    data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    // Get context.
    let context: AppContext = request.guard().await.unwrap();

    // Get grade from post parameter.
    type MyForm = PConForm<(PCon<String, WritePolicy>, PCon<u64, WritePolicy>)>;
    let (user, grade) = MyForm::from_data(request, data).await.unwrap().into_inner();

    // Post them!
    let db: &State<Arc<Mutex<DB>>> = request.guard().await.unwrap();
    let mut db = db.lock().unwrap();
    let result = db.insert(user, grade, context);
    let result = result.map(|_| "success");
    drop(db);

    PConResponseOutcome::from(request, result)
}

// Read a grade: for the signed in user.
pub async fn read_grades<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    // Get context.
    let context: AppContext = request.guard().await.unwrap();
    let db: &State<Arc<Mutex<DB>>> = request.guard().await.unwrap();
    let mut db = db.lock().unwrap();

    // Get user from cookie.
    let user: PConCookie<AuthenticationCookiePolicy> = request.cookies().get("user").unwrap();
    let user: PCon<String, AuthenticationCookiePolicy> = user.value().to_owned_policy().into_pcon();

    // Get grade from post parameter.
    let grades = db.read_by_user(user, context.clone());
    let grades = HashMap::from([("grades", grades)]);
    drop(db);

    PConResponseOutcome::from(request, PConTemplate::render("grades", &grades, context))
}

// Post a grade: for the signed in user.
pub async fn read_all_grades<'a, 'r>(
    request: PConRequest<'a, 'r>,
    _data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    // Get context.
    let context: AppContext = request.guard().await.unwrap();
    let db: &State<Arc<Mutex<DB>>> = request.guard().await.unwrap();
    let mut db = db.lock().unwrap();

    // Get grade from post parameter.
    let grades = db.read_all(context.clone());
    let grades = HashMap::from([("grades", grades)]);
    drop(db);

    PConResponseOutcome::from(request, PConTemplate::render("grades", &grades, context))
}
