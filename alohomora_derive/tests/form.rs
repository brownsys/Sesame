use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy, Reason};

use rocket::http::{ContentType, Cookie, Status};
use rocket::Request;
use alohomora::context::UnprotectedContext;
use alohomora::rocket::{BBoxData, BBoxForm, BBoxRequest, BBoxResponseOutcome, BBoxRocket};
use alohomora::test_route;
use alohomora::testing::{BBoxClient, TestPolicy};
use alohomora_derive::FromBBoxForm;

#[derive(Clone)]
pub struct ExamplePolicy {
    pub cookie: String,
    pub param: String,
}
impl Policy for ExamplePolicy {
    fn name(&self) -> String {
        String::from("ExamplePolicy")
    }
    fn check(&self, _: &UnprotectedContext, _: Reason) -> bool {
        true
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
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
    fn from_cookie<'a, 'r>(_name: &str, _cookie: &'a Cookie<'static>, request: &'a Request<'r>) -> Self {
        ExamplePolicy {
            cookie: request.cookies().get("cookie").unwrap().value().into(),
            param: request.param(0).unwrap().unwrap(),
        }
    }
}

#[derive(FromBBoxForm)]
pub struct Nested {
    pub inner: alohomora::bbox::BBox<String, TestPolicy<ExamplePolicy>>,
    pub vec: Vec<alohomora::bbox::BBox<usize, TestPolicy<ExamplePolicy>>>,
}

#[derive(FromBBoxForm)]
pub struct Simple {
    pub f1: alohomora::bbox::BBox<String, TestPolicy<ExamplePolicy>>,
    pub f2: Nested,
    pub f3: alohomora::bbox::BBox<u8, TestPolicy<ExamplePolicy>>,
}

// Test route.
pub async fn route<'a, 'r>(request: BBoxRequest<'a, 'r>, data: BBoxData<'a>) -> BBoxResponseOutcome<'a> {
    use alohomora::rocket::FromBBoxData;
    type MyForm = BBoxForm<Simple>;
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
        assert_eq!(*form.f2.vec[i].as_ref().discard_box(), i+100);
    }

    BBoxResponseOutcome::from(request, "success")
}

#[test]
fn form_test() {
// Create a rocket instance and mount route.
    let rocket = BBoxRocket::build()
        .mount(
            "/",
            vec![test_route!(Post, "/<user>", route)]
        );

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client.post("/user")
        .header(ContentType::Form)
        .cookie(Cookie::new("cookie", "cvalue"))
        .body("f1=hello&f2.inner=bye&f2.vec.0=100&f2.vec.1=101&f2.vec.2=102&f3=10")
        .dispatch();

    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("success"));
}
