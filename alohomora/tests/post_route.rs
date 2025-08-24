use alohomora::policy::{FrontendPolicy, Join, Policy, Reason};

use alohomora::bbox::BBox;
use alohomora::context::{Context, UnprotectedContext};
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{
    BBoxData, BBoxForm, BBoxRequest, BBoxResponseOutcome, BBoxRocket, ContextResponse, FromBBoxData,
};
use alohomora::test_route;
use alohomora::testing::{BBoxClient, TestPolicy};
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

pub async fn route<'a, 'r>(
    request: BBoxRequest<'a, 'r>,
    data: BBoxData<'a>,
) -> BBoxResponseOutcome<'a> {
    type MyForm = BBoxForm<BBox<String, TestPolicy<UserPolicy>>>;
    let form = MyForm::from_data(request, data).await;
    let param = form.unwrap().into_inner();
    assert_eq!(param.as_ref().discard_box(), "hello");

    let param = param.into_ppr(PrivacyPureRegion::new(|v| format!("Result is {}", &v)));

    BBoxResponseOutcome::from(request, ContextResponse::from((param, Context::test(()))))
}

#[test]
fn test_post() {
    // Create a rocket instance and mount route.
    let rocket = BBoxRocket::build().mount("/", vec![test_route!(Post, "/", route)]);

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client
        .post("/")
        .header(ContentType::Form)
        .cookie(Cookie::new("user", "Kinan"))
        .body("param=hello")
        .dispatch();

    assert_eq!(response.status(), Status::new(200));
    assert_eq!(
        response.into_string().unwrap(),
        String::from("Result is hello")
    );
}

#[test]
fn test_post_failed_policy() {
    // Create a rocket instance and mount route.
    let rocket = BBoxRocket::build().mount("/", vec![test_route!(Post, "/", route)]);

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client
        .post("/")
        .header(ContentType::Form)
        .cookie(Cookie::new("user", "Unauthorized user"))
        .body("param=hello")
        .dispatch();

    assert_eq!(response.status(), Status::new(555));
}
