use std::collections::HashMap;

use alohomora::AlohomoraType;
use alohomora::context::Context;
use alohomora::policy::NoPolicy;
use alohomora_derive::{route, routes, FromBBoxForm, AlohomoraType};

// POST request data.
#[derive(FromBBoxForm, PartialEq, Debug)]
pub struct Nested {
    pub inner: alohomora::bbox::BBox<String, NoPolicy>,
    pub vec: Vec<alohomora::bbox::BBox<usize, NoPolicy>>,
}

#[derive(FromBBoxForm, PartialEq, Debug)]
pub struct Simple {
    pub f1: alohomora::bbox::BBox<String, NoPolicy>,
    pub f2: Nested,
    pub f3: alohomora::bbox::BBox<u8, NoPolicy>,
    pub f4: HashMap<String, alohomora::bbox::BBox<u8, NoPolicy>>,
}

// Guard managed by rocket.
struct Config(pub String);

// Get request param.
#[derive(FromBBoxForm)]
struct Dog {
    pub name: alohomora::bbox::BBox<String, NoPolicy>,
    pub age: alohomora::bbox::BBox<usize, NoPolicy>,
}

// Context derived from both request and also form data.
#[derive(AlohomoraType)]
struct ContextData {
  // we acquire this from the post data via BBoxForm<Simple> (also would work
  // had post data been BBoxJson<Simple> etc).
  pub f1: alohomora::bbox::BBox<String, NoPolicy>,
  // we acquire this a cookie via BBoxRequest.
  pub cookie: alohomora::bbox::BBox<String, NoPolicy>,
}

// Notice that we need to include *BBoxForm<Simple>* (or BBoxJson<Simple>) in
// the trait generics, and NOT just Simple.
#[rocket::async_trait]
impl<'a, 'r: 'a> alohomora::rocket::FromBBoxRequestAndData<'a, 'r, alohomora::rocket::BBoxForm<Simple>> for ContextData {
    type BBoxError = ();
    async fn from_bbox_request_and_data(
        request: alohomora::rocket::BBoxRequest<'a, 'r>,
        form: &'_ alohomora::rocket::BBoxForm<Simple>,
    ) -> alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
        alohomora::rocket::BBoxRequestOutcome::Success(ContextData {
            f1: form.f1.clone(),
            cookie: request.cookies().get("mycookie").unwrap().value().to_owned(),
        })
    }
}

// HTTP request.
// POST /route/<num>?<dog>&<a>
// Example: /route/5?a=apple&dog.name=Max&dog.age=10
#[route(POST, "/route/<num>?<dog>&<a>", data = "<data>", with_data = "<context>")]
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
        f1: alohomora::bbox::BBox::new(String::from("hello"), NoPolicy {}),
        f2: Nested {
            inner: alohomora::bbox::BBox::new(String::from("bye"), NoPolicy {}),
            vec: vec![
                alohomora::bbox::BBox::new(100, NoPolicy {}),
                alohomora::bbox::BBox::new(200, NoPolicy {})
            ],
        },
        f3: alohomora::bbox::BBox::new(55, NoPolicy {}),
        f4: HashMap::from([
            (String::from("k1"), alohomora::bbox::BBox::new(11, NoPolicy {})),
            (String::from("k2"), alohomora::bbox::BBox::new(12, NoPolicy {})),
        ]),
    };

    assert_eq!(data.into_inner(), simple);

    assert_eq!(context.data().unwrap().f1.as_ref().discard_box(), "hello");
    assert_eq!(context.data().unwrap().cookie.as_ref().discard_box(), "cookie value!");

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
        .cookie(rocket::http::Cookie::new("mycookie", "cookie value!"))
        .header(rocket::http::ContentType::Form)
        .body("f1=hello&f2.inner=bye&f2.vec.0=100&f2.vec.1=200&f3=55&f4.k1=11&f4.k2=12")
        .dispatch();

    assert_eq!(response.status(), rocket::http::Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("apple"));
}
