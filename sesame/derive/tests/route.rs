use std::collections::HashMap;

use sesame::context::Context;
use sesame::policy::NoPolicy;

use sesame_derive::{route, routes, FromBBoxForm, SesameType};

// POST request data.
#[derive(FromBBoxForm, PartialEq, Debug)]
pub struct Nested {
    pub inner: sesame::bbox::BBox<String, NoPolicy>,
    pub vec: Vec<sesame::bbox::BBox<usize, NoPolicy>>,
}

#[derive(FromBBoxForm, PartialEq, Debug)]
pub struct Simple {
    pub f1: sesame::bbox::BBox<String, NoPolicy>,
    pub f2: Nested,
    pub f3: sesame::bbox::BBox<u8, NoPolicy>,
    pub f4: HashMap<String, sesame::bbox::BBox<u8, NoPolicy>>,
}

// Guard managed by rocket.
struct Config(pub String);

// Get request param.
#[derive(FromBBoxForm)]
struct Dog {
    pub name: sesame::bbox::BBox<String, NoPolicy>,
    pub age: sesame::bbox::BBox<usize, NoPolicy>,
}

// Context derived from both request and also form data.
#[derive(SesameType)]
struct ContextData {
    // we acquire this from the post data via BBoxForm<Simple> (also would work
    // had post data been BBoxJson<Simple> etc).
    pub f1: sesame::bbox::BBox<String, NoPolicy>,
    // we acquire this a cookie via BBoxRequest.
    pub cookie: sesame::bbox::BBox<String, NoPolicy>,
}

// Notice that we need to include *BBoxForm<Simple>* (or BBoxJson<Simple>) in
// the trait generics, and NOT just Simple.
#[rocket::async_trait]
impl<'a, 'r: 'a>
    sesame_rocket::rocket::FromBBoxRequestAndData<'a, 'r, sesame_rocket::rocket::BBoxForm<Simple>>
    for ContextData
{
    type BBoxError = ();
    async fn from_bbox_request_and_data(
        request: sesame_rocket::rocket::BBoxRequest<'a, 'r>,
        form: &'_ sesame_rocket::rocket::BBoxForm<Simple>,
    ) -> sesame_rocket::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
        sesame_rocket::rocket::BBoxRequestOutcome::Success(ContextData {
            f1: form.f1.clone(),
            cookie: request
                .cookies()
                .get("mycookie")
                .unwrap()
                .value()
                .to_owned(),
        })
    }
}

// HTTP request.
// POST /route/<num>?<dog>&<a>
// Example: /route/5?a=apple&dog.name=Max&dog.age=10
#[route(
    POST,
    "/route/<num>?<dog>&<a>",
    data = "<data>",
    with_data = "<context>"
)]
async fn my_route(
    config: &rocket::State<Config>,
    context: Context<ContextData>,
    num: sesame::bbox::BBox<u8, NoPolicy>,
    a: sesame::bbox::BBox<String, NoPolicy>,
    dog: Dog,
    data: sesame_rocket::rocket::BBoxForm<Simple>,
) -> sesame_rocket::rocket::ContextResponse<String, NoPolicy, ContextData> {
    // Ensure things got parsed/created correctly.
    assert_eq!(config.0, "myconfig");
    assert_eq!(context.route(), "/route/<num>?<dog>&<a>");
    assert_eq!(*num.as_ref().discard_box(), 5);
    assert_eq!(a.as_ref().discard_box(), "apple");
    assert_eq!(dog.name.as_ref().discard_box(), "Max");
    assert_eq!(*dog.age.as_ref().discard_box(), 10);

    let simple = Simple {
        f1: sesame::bbox::BBox::new(String::from("hello"), NoPolicy {}),
        f2: Nested {
            inner: sesame::bbox::BBox::new(String::from("bye"), NoPolicy {}),
            vec: vec![
                sesame::bbox::BBox::new(100, NoPolicy {}),
                sesame::bbox::BBox::new(200, NoPolicy {}),
            ],
        },
        f3: sesame::bbox::BBox::new(55, NoPolicy {}),
        f4: HashMap::from([
            (
                String::from("k1"),
                sesame::bbox::BBox::new(11, NoPolicy {}),
            ),
            (
                String::from("k2"),
                sesame::bbox::BBox::new(12, NoPolicy {}),
            ),
        ]),
    };

    assert_eq!(data.into_inner(), simple);

    assert_eq!(context.data().unwrap().f1.as_ref().discard_box(), "hello");
    assert_eq!(
        context.data().unwrap().cookie.as_ref().discard_box(),
        "cookie value!"
    );

    // all good.
    sesame_rocket::rocket::ContextResponse::from((a, context))
}

#[test]
fn simple_from_bbox_form_test() {
    let rocket = sesame_rocket::rocket::BBoxRocket::<::rocket::Build>::build()
        .manage(Config(String::from("myconfig")))
        .mount("/", routes![my_route]);

    // Create a client.
    let client = sesame_rocket::testing::BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client
        .post("/route/5?a=apple&dog.name=Max&dog.age=10")
        .cookie(rocket::http::Cookie::new("mycookie", "cookie value!"))
        .header(rocket::http::ContentType::Form)
        .body("f1=hello&f2.inner=bye&f2.vec.0=100&f2.vec.1=200&f3=55&f4.k1=11&f4.k2=12")
        .dispatch();

    assert_eq!(response.status(), rocket::http::Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("apple"));
}
