use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rocket::State;
use alohomora::bbox::BBox;
use alohomora::rocket::{BBoxCookie, BBoxData, BBoxForm, BBoxRequest, BBoxResponseOutcome, BBoxTemplate, FromBBoxData};
use crate::application::context::AppContext;
use crate::application::db::DB;
use crate::application::policy::{AuthenticationCookiePolicy, WritePolicy};

// Logins in as a user.
pub async fn login<'a, 'r>(request: BBoxRequest<'a, 'r>, _data: BBoxData<'a>) -> BBoxResponseOutcome<'a> {
    let context: AppContext = request.guard().await.unwrap();

    let username: BBox<String, AuthenticationCookiePolicy> = request.param(1).unwrap().unwrap();
    request.cookies().add(BBoxCookie::new("user", username), context).unwrap();

    BBoxResponseOutcome::from(request, "success")
}

// Post a grade: have to be admin.
pub async fn post_grade<'a, 'r>(request: BBoxRequest<'a, 'r>, data: BBoxData<'a>) -> BBoxResponseOutcome<'a> {
    // Get context.
    let context: AppContext = request.guard().await.unwrap();

    // Get grade from post parameter.
    type MyForm = BBoxForm<(BBox<String, WritePolicy>, BBox<u64, WritePolicy>)>;
    let (user, grade) =
        MyForm::from_data(request, data).await.unwrap().into_inner();

    // Post them!
    let db: &State<Arc<Mutex<DB>>> = request.guard().await.unwrap();
    let mut db = db.lock().unwrap();
    let result = db.insert(user, grade, context);
    drop(db);

    result.unwrap();
    BBoxResponseOutcome::from(request, "success")
}


// Read a grade: for the signed in user.
pub async fn read_grades<'a, 'r>(request: BBoxRequest<'a, 'r>, _data: BBoxData<'a>) -> BBoxResponseOutcome<'a> {
    // Get context.
    let context: AppContext = request.guard().await.unwrap();
    let db: &State<Arc<Mutex<DB>>> = request.guard().await.unwrap();
    let mut db = db.lock().unwrap();

    // Get user from cookie.
    let user: BBoxCookie<AuthenticationCookiePolicy> = request.cookies().get("user").unwrap();
    let user: BBox<String, AuthenticationCookiePolicy> = user.value().to_owned_policy().into_bbox();

    // Get grade from post parameter.
    let grades = db.read_by_user(user, context.clone());
    let grades = HashMap::from([("grades", grades)]);
    drop(db);

    BBoxResponseOutcome::from(request, BBoxTemplate::render("grades", &grades, context))
}

// Post a grade: for the signed in user.
pub async fn read_all_grades<'a, 'r>(request: BBoxRequest<'a, 'r>, _data: BBoxData<'a>) -> BBoxResponseOutcome<'a> {
    // Get context.
    let context: AppContext = request.guard().await.unwrap();
    let db: &State<Arc<Mutex<DB>>> = request.guard().await.unwrap();
    let mut db = db.lock().unwrap();

    // Get grade from post parameter.
    let grades = db.read_all(context.clone());
    let grades = HashMap::from([("grades", grades)]);
    drop(db);

    BBoxResponseOutcome::from(request, BBoxTemplate::render("grades", &grades, context))
}