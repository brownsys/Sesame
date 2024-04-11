use std::collections::HashMap;
use alohomora::policy::{AnyPolicy, FrontendPolicy, NoPolicy, Policy, Reason};

use rocket::http::{ContentType, Cookie, Status};
use rocket::Request;
use alohomora::bbox::BBox;
use alohomora::context::{Context, UnprotectedContext};
use alohomora::rocket::{BBoxData, BBoxJson, BBoxRequest, BBoxResponseOutcome, BBoxRocket, FromBBoxData, InputBBoxValue, JsonResponse, OutputBBoxValue, RequestBBoxJson, ResponseBBoxJson};
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

// Parses request bodies.
struct MyJsonData {
    pub id: BBox<u64, TestPolicy<UserPolicy>>,
    pub email: BBox<String, TestPolicy<NoPolicy>>,
}
impl RequestBBoxJson for MyJsonData {
    fn from_json(mut value: InputBBoxValue, request: BBoxRequest<'_, '_>) -> Result<Self, &'static str> {
        Ok(MyJsonData {
            id: value.get("id")?.into_json(request)?,
            email: value.get("email")?.into_json(request)?,
        })
    }
}
impl ResponseBBoxJson for MyJsonData {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::Object(HashMap::from([
            (String::from("id"), self.id.to_json()),
            (String::from("email"), self.email.to_json()),
        ]))
    }
}

pub async fn route<'a, 'r>(request: BBoxRequest<'a, 'r>, data: BBoxData<'a>) -> BBoxResponseOutcome<'a> {
    let json = BBoxJson::<MyJsonData>::from_data(request, data).await;
    let json = json.unwrap().into_inner();
    assert_eq!(*json.id.as_ref().discard_box(), 100);
    assert_eq!(json.email.as_ref().discard_box(), "email@email.com");

    let response = MyJsonData {
        id: BBox::new(250, json.id.policy().clone()),
        email: BBox::new(String::from("email@response.com"), json.email.policy().clone()),
    };

    BBoxResponseOutcome::from(request, JsonResponse::from((response, Context::test(()))))
}

#[test]
fn test_json() {
    // Create a rocket instance and mount route.
    let rocket = BBoxRocket::build()
        .mount(
            "/",
            vec![test_route!(Post, "/", route)]
        );

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client.post("/")
        .header(ContentType::JSON)
        .cookie(Cookie::new("user", "Kinan"))
        .body("{\"id\": 100, \"email\": \"email@email.com\"}")
        .dispatch();

    assert_eq!(response.status(), Status::new(200));

    let json: serde_json::Value = response.into_json().unwrap();
    assert_eq!(json.get("id").unwrap().as_u64(), Some(250));
    assert_eq!(json.get("email").unwrap().as_str(), Some("email@response.com"));
}

#[test]
fn test_json_failed_policy() {
    // Create a rocket instance and mount route.
    let rocket = BBoxRocket::build()
        .mount(
            "/",
            vec![test_route!(Post, "/", route)]
        );

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client.post("/")
        .header(ContentType::JSON)
        .cookie(Cookie::new("user", "Artem"))
        .body("{\"id\": 100, \"email\": \"email@email.com\"}")
        .dispatch();

    assert_eq!(response.status(), Status::new(555));
}