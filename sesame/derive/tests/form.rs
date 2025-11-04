use sesame::context::UnprotectedContext;
use sesame::policy::{Reason, SimplePolicy};
use sesame::testing::TestPolicy;
use sesame_derive::FromPConForm;
use sesame_rocket::policy::FrontendPolicy;
use sesame_rocket::rocket::{PConData, PConForm, PConRequest, PConResponseOutcome, SesameRocket};
use sesame_rocket::test_route;
use sesame_rocket::testing::SesameClient;

use rocket::http::{ContentType, Cookie, Status};
use rocket::Request;

#[derive(Clone)]
pub struct ExamplePolicy {
    pub cookie: String,
    pub param: String,
}
impl SimplePolicy for ExamplePolicy {
    fn simple_name(&self) -> String {
        String::from("ExamplePolicy")
    }
    fn simple_check(&self, _: &UnprotectedContext, _: Reason) -> bool {
        true
    }
    fn simple_join_direct(&mut self, _other: &mut Self) {
        todo!()
    }
}
impl FrontendPolicy for ExamplePolicy {
    fn from_request(request: &'_ Request<'_>) -> Self {
        ExamplePolicy {
            cookie: request.cookies().get("cookie").unwrap().value().into(),
            param: request.param(0).unwrap().unwrap(),
        }
    }
    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        request: &'a Request<'r>,
    ) -> Self {
        ExamplePolicy {
            cookie: request.cookies().get("cookie").unwrap().value().into(),
            param: request.param(0).unwrap().unwrap(),
        }
    }
}

#[derive(FromPConForm)]
pub struct Nested {
    pub inner: sesame::pcon::PCon<String, TestPolicy<ExamplePolicy>>,
    pub vec: Vec<sesame::pcon::PCon<usize, TestPolicy<ExamplePolicy>>>,
}

#[derive(FromPConForm)]
pub struct Simple {
    pub f1: sesame::pcon::PCon<String, TestPolicy<ExamplePolicy>>,
    pub f2: Nested,
    pub f3: sesame::pcon::PCon<u8, TestPolicy<ExamplePolicy>>,
}

// Test route.
pub async fn route<'a, 'r>(
    request: PConRequest<'a, 'r>,
    data: PConData<'a>,
) -> PConResponseOutcome<'a> {
    use sesame_rocket::rocket::FromPConData;
    type MyForm = PConForm<Simple>;
    let form = MyForm::from_data(request, data).await.unwrap().into_inner();

    assert_eq!(form.f1.policy().policy().cookie, "cvalue");
    assert_eq!(form.f1.policy().policy().param, "user");
    assert_eq!(form.f2.inner.policy().policy().cookie, "cvalue");
    assert_eq!(form.f2.inner.policy().policy().param, "user");
    assert_eq!(form.f3.policy().policy().cookie, "cvalue");
    assert_eq!(form.f3.policy().policy().param, "user");

    assert_eq!(form.f1.as_ref().discard_box(), "hello");
    assert_eq!(form.f2.inner.as_ref().discard_box(), "bye");
    assert_eq!(*form.f3.as_ref().discard_box(), 10);

    for i in 0..form.f2.vec.len() {
        assert_eq!(form.f2.vec[i].policy().policy().cookie, "cvalue");
        assert_eq!(form.f2.vec[i].policy().policy().param, "user");
        assert_eq!(*form.f2.vec[i].as_ref().discard_box(), i + 100);
    }

    PConResponseOutcome::from(request, "success")
}

#[test]
fn form_test() {
    // Create a rocket instance and mount route.
    let rocket = SesameRocket::build().mount("/", vec![test_route!(Post, "/<user>", route)]);

    // Create a client.
    let client = SesameClient::tracked(rocket).expect("valid `Rocket`");
    let response = client
        .post("/user")
        .header(ContentType::Form)
        .cookie(Cookie::new("cookie", "cvalue"))
        .body("f1=hello&f2.inner=bye&f2.vec.0=100&f2.vec.1=101&f2.vec.2=102&f3=10")
        .dispatch();

    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("success"));
}
