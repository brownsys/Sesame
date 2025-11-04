use std::collections::HashMap;

use sesame::context::{Context, UnprotectedContext};
use sesame::pcon::PCon;
use sesame::policy::{Join, NoPolicy, Policy, Reason};
use sesame::testing::TestPolicy;

use sesame_rocket::policy::FrontendPolicy;
use sesame_rocket::rocket::{
    FromPConData, InputPConValue, JsonResponse, OutputPConValue, PConData, PConJson, PConRequest,
    PConResponseOutcome, RequestPConJson, ResponsePConJson, SesameRocket,
};
use sesame_rocket::test_route;
use sesame_rocket::testing::SesameClient;

use rocket::http::{ContentType, Cookie, Status};
use rocket::Request;

#[derive(Clone)]
pub struct UserPolicy {
    pub name: String,
}
impl Join for UserPolicy {}
impl Policy for UserPolicy {
    fn name(&self) -> String {
        String::from("UserPolicy")
    }
    fn check(&self, _: &UnprotectedContext, _: Reason) -> bool {
        self.name == String::from("Kinan")
    }
}
impl FrontendPolicy for UserPolicy {
    fn from_request(request: &'_ Request<'_>) -> Self {
        let user = request.cookies().get("user").unwrap();
        UserPolicy {
            name: String::from(user.value()),
        }
    }
    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        request: &'a Request<'r>,
    ) -> Self {
        let user = request.cookies().get("user").unwrap();
        UserPolicy {
            name: String::from(user.value()),
        }
    }
}

// Parses request bodies.
struct MyJsonData {
    pub id: PCon<u64, TestPolicy<UserPolicy>>,
    pub email: PCon<String, TestPolicy<NoPolicy>>,
}
impl RequestPConJson for MyJsonData {
    fn from_json(
        mut value: InputPConValue,
        request: PConRequest<'_, '_>,
    ) -> Result<Self, &'static str> {
        Ok(MyJsonData {
            id: value.get("id")?.into_json(request)?,
            email: value.get("email")?.into_json(request)?,
        })
    }
}
impl ResponsePConJson for MyJsonData {
    fn to_json(self) -> OutputPConValue {
        OutputPConValue::Object(HashMap::from([
            (String::from("id"), self.id.to_json()),
            (String::from("email"), self.email.to_json()),
        ]))
    }
}

pub async fn route<'a, 'r>(
    request: PConRequest<'a, 'r>,
    data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    let json = PConJson::<MyJsonData>::from_data(request, data).await;
    let json = json.unwrap().into_inner();
    assert_eq!(*json.id.as_ref().discard_box(), 100);
    assert_eq!(json.email.as_ref().discard_box(), "email@email.com");

    let response = MyJsonData {
        id: PCon::new(250, json.id.policy().clone()),
        email: PCon::new(
            String::from("email@response.com"),
            json.email.policy().clone(),
        ),
    };

    PConResponseOutcome::from(request, JsonResponse::from((response, Context::test(()))))
}

#[test]
fn test_json() {
    // Create a rocket instance and mount route.
    let rocket = SesameRocket::build().mount("/", vec![test_route!(Post, "/", route)]);

    // Create a client.
    let client = SesameClient::tracked(rocket).expect("valid `Rocket`");
    let response = client
        .post("/")
        .header(ContentType::JSON)
        .cookie(Cookie::new("user", "Kinan"))
        .body("{\"id\": 100, \"email\": \"email@email.com\"}")
        .dispatch();

    assert_eq!(response.status(), Status::new(200));

    let json: serde_json::Value = response.into_json().unwrap();
    assert_eq!(json.get("id").unwrap().as_u64(), Some(250));
    assert_eq!(
        json.get("email").unwrap().as_str(),
        Some("email@response.com")
    );
}

#[test]
fn test_json_failed_policy() {
    // Create a rocket instance and mount route.
    let rocket = SesameRocket::build().mount("/", vec![test_route!(Post, "/", route)]);

    // Create a client.
    let client = SesameClient::tracked(rocket).expect("valid `Rocket`");
    let response = client
        .post("/")
        .header(ContentType::JSON)
        .cookie(Cookie::new("user", "Artem"))
        .body("{\"id\": 100, \"email\": \"email@email.com\"}")
        .dispatch();

    assert_eq!(response.status(), Status::new(491));
}
