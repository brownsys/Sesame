use std::collections::HashMap;

use serde::Serialize;

use alohomora::context::Context;
use alohomora::policy::NoPolicy;
use alohomora::testing::{TestContextData, TestPolicy};
use alohomora_derive::{route, routes, RequestBBoxJson, ResponseBBoxJson};

// POST request data.
#[derive(RequestBBoxJson, ResponseBBoxJson, PartialEq, Debug)]
pub struct Nested {
    pub inner: alohomora::bbox::BBox<String, TestPolicy<NoPolicy>>,
    pub vec: Vec<alohomora::bbox::BBox<i64, TestPolicy<NoPolicy>>>,
}

#[derive(Serialize)]
pub struct Nested2 {
    pub f1: String,
    pub f2: f64,
}

#[derive(RequestBBoxJson, PartialEq, Debug)]
pub struct Simple {
    pub f1: alohomora::bbox::BBox<String, TestPolicy<NoPolicy>>,
    pub f2: Nested,
    pub f3: alohomora::bbox::BBox<u64, TestPolicy<NoPolicy>>,
    pub f4: HashMap<String, alohomora::bbox::BBox<u64, TestPolicy<NoPolicy>>>,
    pub f5: Option<alohomora::bbox::BBox<u64, TestPolicy<NoPolicy>>>,
}

#[derive(ResponseBBoxJson)]
#[response_bbox_json(as_is = [f3])]
pub struct Output {
    pub f1: alohomora::bbox::BBox<String, TestPolicy<NoPolicy>>,
    pub f2: Nested,
    pub f3: Nested2,
    pub f4: Option<Nested>,
    pub f5: String,
}

#[route(POST, "/", data = "<data>")]
fn my_route(
    // In reality, we would receive context as a parameter, commented out to simplify testing.
    // context: Context<()>,
    data: alohomora::rocket::BBoxJson<Simple>,
) -> alohomora::rocket::JsonResponse<Output, TestContextData<()>> {
    let simple = Simple {
        f1: alohomora::bbox::BBox::new(String::from("hello"), TestPolicy::new(NoPolicy {})),
        f2: Nested {
            inner: alohomora::bbox::BBox::new(String::from("bye"), TestPolicy::new(NoPolicy {})),
            vec: vec![
                alohomora::bbox::BBox::new(-100, TestPolicy::new(NoPolicy {})),
                alohomora::bbox::BBox::new(200, TestPolicy::new(NoPolicy {}))
            ],
        },
        f3: alohomora::bbox::BBox::new(55, TestPolicy::new(NoPolicy {})),
        f4: HashMap::from([
            (String::from("k1"), alohomora::bbox::BBox::new(11, TestPolicy::new(NoPolicy {}))),
            (String::from("k2"), alohomora::bbox::BBox::new(12, TestPolicy::new(NoPolicy {}))),
        ]),
        f5: None,
    };

    // Check that input is parsed correctly.
    assert_eq!(data.into_inner(), simple);

    let output = Output {
        f1: alohomora::bbox::BBox::new(String::from("hi"), TestPolicy::new(NoPolicy {})),
        f2: simple.f2,
        f3: Nested2 {
            f1: String::from("nestedf1"),
            f2: 22.5,
        },
        f4: None,
        f5: String::from("raw"),
    };

    println!("test");
    // Return result.
    alohomora::rocket::JsonResponse::from((output, Context::test(())))
}

#[test]
fn simple_from_bbox_form_test() {
    let rocket = alohomora::rocket::BBoxRocket::<::rocket::Build>::build()
        .mount("/", routes![my_route]);

    // Create a client.
    let client = alohomora::testing::BBoxClient::tracked(rocket).expect("valid `Rocket`");
    let response = client.post("/")
        .header(rocket::http::ContentType::JSON)
        .body("{\
            \"f1\": \"hello\",\
            \"f2\": {\
                \"inner\": \"bye\",
                \"vec\": [-100, 200]
            },\
            \"f3\": 55,\
            \"f4\": {\
              \"k1\": 11,
               \"k2\": 12
            }\
        }")
        .dispatch();

    // Validate response.
    use std::iter::FromIterator;
    use serde_json::{Map, Value, Number};
    assert_eq!(response.status(), rocket::http::Status::new(200));

    let response: Value = response.into_json().unwrap();
    assert_eq!(
        response,
        Value::Object(Map::from_iter([
            (String::from("f1"), Value::String(String::from("hi"))),
            (String::from("f2"), Value::Object(Map::from_iter([
                (String::from("inner"), Value::String(String::from("bye"))),
                (String::from("vec"), Value::Array(vec![
                    Value::Number(Number::from(-100i64)),
                    Value::Number(Number::from(200i64)),
                ])),
            ]))),
            (String::from("f3"), Value::Object(Map::from_iter([
                (String::from("f1"), Value::String(String::from("nestedf1"))),
                (String::from("f2"), Value::Number(Number::from_f64(22.5f64).unwrap())),
            ]))),
            (String::from("f4"), Value::Null),
            (String::from("f5"), Value::String(String::from("raw"))),
        ]))
    );
}