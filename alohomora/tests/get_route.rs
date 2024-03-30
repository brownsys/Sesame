use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy, Reason};

use rocket::http::{ContentType, Cookie, Status};
use rocket::Request;
use alohomora::bbox::BBox;
use alohomora::context::{Context, UnprotectedContext};
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{BBoxCookie, BBoxData, BBoxRequest, BBoxResponseOutcome, BBoxRocket, ContextResponse, FromBBoxFormField};
use alohomora::test_route;
use alohomora::testing::{BBoxClient, TestPolicy};

#[derive(Clone)]
pub struct UserPolicy {
    pub name: String,
}
impl Policy for UserPolicy {
    fn name(&self) -> String {
        String::from("UserPolicy")
    }
    fn check(&self, _: &UnprotectedContext, _: Reason) -> bool {
        self.name == String::from("Kinan")
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl FrontendPolicy for UserPolicy {
    fn from_request(request: &'_ Request<'_>) -> Self {
        let user = request.cookies().get("user").unwrap();
        UserPolicy { name: String::from(user.value()) }
    }
    fn from_cookie<'a, 'r>(_name: &str, _cookie: &'a Cookie<'static>, request: &'a Request<'r>) -> Self {
        let user = request.cookies().get("user").unwrap();
        UserPolicy { name: String::from(user.value()) }
    }
}

#[derive(Clone)]
pub struct HardcodedPolicy(pub bool);
impl Policy for HardcodedPolicy {
    fn name(&self) -> String {
        String::from("HardcodedPolicy")
    }
    fn check(&self, _: &UnprotectedContext, _: Reason) -> bool {
        self.0
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl FrontendPolicy for HardcodedPolicy {
    fn from_request<'a, 'r>(_request: &'a Request<'r>) -> Self where Self: Sized { todo!() }
    fn from_cookie<'a, 'r>(_name: &str, _cookie: &'a Cookie<'static>, _request: &'a Request<'r>) -> Self
        where Self: Sized
    { todo!() }
}

pub async fn route<'a, 'r>(request: BBoxRequest<'a, 'r>, _data: BBoxData<'a>) -> BBoxResponseOutcome<'a> {
    // Look at managed guarded, e.g. config data.
    let guard: &rocket::State<String> = request.guard().await.unwrap();

    // Read cookie.
    let cookie = request.cookies().get("mycookie").unwrap();
    let cookie: BBox<String, TestPolicy<UserPolicy>> = BBox::from(cookie);

    // Read parameters that are part of the path.
    let num: BBox<u32, TestPolicy<UserPolicy>> = request.param(1).unwrap().unwrap();
    let string: BBox<String, TestPolicy<UserPolicy>> = request.param(2).unwrap().unwrap();

    // Read get parameters after ?
    let mut a: Option<BBox<String, TestPolicy<UserPolicy>>> = None;
    let mut b_a: Option<BBox<u32, TestPolicy<UserPolicy>>> = None;
    let mut b_b: Option<BBox<u32, TestPolicy<UserPolicy>>> = None;
    for f in request.query_fields() {
        let mut name = f.name.clone();
        let key = name.key().unwrap();
        if key == "a" {
            a = Some(BBox::from_bbox_value(f, request).unwrap());
            continue;
        } else if key == "b" {
            name.shift();
            let key = name.key().unwrap();
            if key == "a" {
                b_a = Some(BBox::from_bbox_value(f, request).unwrap());
                continue;
            } else if key == "b" {
                b_b = Some(BBox::from_bbox_value(f, request).unwrap());
                continue;
            }
        }
        panic!("bad key");
    }

    assert_eq!(guard.inner(), "MyGuard");
    assert_eq!(cookie.as_ref().discard_box(), "myvalue");
    assert_eq!(*num.as_ref().discard_box(), 10);
    assert_eq!(string.as_ref().discard_box(), "hello");
    assert_eq!(a.unwrap().as_ref().discard_box(), "A");
    assert_eq!(*b_a.unwrap().as_ref().discard_box(), 1);
    assert_eq!(*b_b.unwrap().as_ref().discard_box(), 2);

    //TODO(babman): test template rendering, and redirect.
    //TODo(babman): finish tests in alohomora_derive.
    // Assign some cookies.
    let cookie1 = BBox::new(String::from("cook"), HardcodedPolicy(true));
    let cookie1 = BBoxCookie::new("good", cookie1);
    request.cookies().add(cookie1, Context::test(())).unwrap();

    let cookie2 = BBox::new(String::from("cook2"), HardcodedPolicy(false));
    let cookie2 = BBoxCookie::new("bad", cookie2);
    assert!(request.cookies().add(cookie2, Context::test(())).is_err());

    // Return post request.
    let param = string.into_ppr(
        PrivacyPureRegion::new(|v| format!("Result is {}", &v))
    );

    BBoxResponseOutcome::from(request, ContextResponse::from((param, Context::test(()))))
}

#[test]
fn test_get() {
    let guard = String::from("MyGuard");

    // Create a rocket instance and mount route.
    let rocket = BBoxRocket::build()
        .manage(guard)
        .mount(
            "/",
            vec![test_route!(Get, "/route/<num>/<string>?<a>&<b>", route)]
        );

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client.get("/route/10/hello?a=A&b.a=1&b.b=2")
        .header(ContentType::Form)
        .cookie(Cookie::new("user", "Kinan"))
        .cookie(Cookie::new("mycookie", "myvalue"))
        .dispatch();

    // Check cookies are correctly set.
    assert_eq!(response.cookies().get("good").unwrap().value(), "cook");
    assert!(response.cookies().get("bad").is_none());

    // Check response body and status are correct.
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("Result is hello"));
}

#[test]
fn test_get_failed_policy() {
    let guard = String::from("MyGuard");

    // Create a rocket instance and mount route.
    let rocket = BBoxRocket::build()
        .manage(guard)
        .mount(
            "/",
            vec![test_route!(Get, "/route/<num>/<string>?<a>&<b>", route)]
        );

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client.get("/route/10/hello?a=A&b.a=1&b.b=2")
        .cookie(Cookie::new("user", "Unauthorized user"))
        .cookie(Cookie::new("mycookie", "myvalue"))
        .dispatch();

    // No cookie is set cause request failed.
    assert!(response.cookies().get("good").is_none());
    assert!(response.cookies().get("bad").is_none());

    // Check response fails.
    assert_eq!(response.status(), Status::new(555));
}