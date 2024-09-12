use std::collections::HashMap;

use alohomora::context::Context;
use alohomora::policy::NoPolicy;
use alohomora::testing::TestPolicy;
use alohomora_derive::{route, routes, FromBBoxForm, AlohomoraType};

// POST request data.
#[derive(FromBBoxForm, PartialEq, Debug)]
pub struct Nested {
    pub inner: alohomora::bbox::BBox<String, TestPolicy<NoPolicy>>,
    pub vec: Vec<alohomora::bbox::BBox<usize, TestPolicy<NoPolicy>>>,
}

#[derive(FromBBoxForm, PartialEq, Debug)]
pub struct Simple {
    pub f1: alohomora::bbox::BBox<String, TestPolicy<NoPolicy>>,
    pub f2: Nested,
    pub f3: alohomora::bbox::BBox<u8, TestPolicy<NoPolicy>>,
    pub f4: HashMap<String, alohomora::bbox::BBox<u8, TestPolicy<NoPolicy>>>,
}

// Guard managed by rocket.
struct Config(pub String);

// Get request param.
#[derive(FromBBoxForm)]
struct Dog {
    pub name: alohomora::bbox::BBox<String, NoPolicy>,
    pub age: alohomora::bbox::BBox<usize, NoPolicy>,
}


#[derive(AlohomoraType)]
struct ContextData {}

#[rocket::async_trait]
impl<'a, 'r> alohomora::rocket::FromBBoxRequest<'a, 'r> for ContextData {
    type BBoxError = ();
    async fn from_bbox_request(_request: alohomora::rocket::BBoxRequest<'a, 'r>) -> alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
        alohomora::rocket::BBoxRequestOutcome::Success(ContextData {})
    }
}

// HTTP request.
// POST /route/<num>?<dog>&<a>
// Example: /route/5?a=apple&dog.name=Max&dog.age=10
#[route(POST, "/route/<num>?<dog>&<a>", data = "<data>")]
async fn my_route(
    config: &rocket::State<Config>,
    context: Context<ContextData>,
    num: alohomora::bbox::BBox<u8, NoPolicy>,
    a: alohomora::bbox::BBox<String, NoPolicy>,
    dog: Dog,
    data: alohomora::rocket::BBoxForm<Simple>,
) -> alohomora::rocket::ContextResponse<String, NoPolicy, ContextData> {
    // Ensure things got parsed/created correctly.
    assert_eq!(config.0, "myconfig");
    assert_eq!(context.route(), "/route/<num>?<dog>&<a>");
    assert_eq!(*num.as_ref().discard_box(), 5);
    assert_eq!(a.as_ref().discard_box(), "apple");
    assert_eq!(dog.name.as_ref().discard_box(), "Max");
    assert_eq!(*dog.age.as_ref().discard_box(), 10);

    let simple = Simple {
        f1: alohomora::bbox::BBox::new(String::from("hello"), TestPolicy::new(NoPolicy {})),
        f2: Nested {
            inner: alohomora::bbox::BBox::new(String::from("bye"), TestPolicy::new(NoPolicy {})),
            vec: vec![
                alohomora::bbox::BBox::new(100, TestPolicy::new(NoPolicy {})),
                alohomora::bbox::BBox::new(200, TestPolicy::new(NoPolicy {}))
            ],
        },
        f3: alohomora::bbox::BBox::new(55, TestPolicy::new(NoPolicy {})),
        f4: HashMap::from([
            (String::from("k1"), alohomora::bbox::BBox::new(11, TestPolicy::new(NoPolicy {}))),
            (String::from("k2"), alohomora::bbox::BBox::new(12, TestPolicy::new(NoPolicy {}))),
        ]),
    };

    assert_eq!(data.into_inner(), simple);

    // all good.
    alohomora::rocket::ContextResponse::from((a, context))
}

#[test]
fn simple_from_bbox_form_test() {
    let rocket = alohomora::rocket::BBoxRocket::<::rocket::Build>::build()
        .manage(Config(String::from("myconfig")))
        .mount("/", routes![my_route]);

    // Create a client.
    let client = alohomora::testing::BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client.post("/route/5?a=apple&dog.name=Max&dog.age=10")
        .header(rocket::http::ContentType::Form)
        .body("f1=hello&f2.inner=bye&f2.vec.0=100&f2.vec.1=200&f3=55&f4.k1=11&f4.k2=12")
        .dispatch();

    assert_eq!(response.status(), rocket::http::Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("apple"));
}
