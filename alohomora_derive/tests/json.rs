use std::collections::HashMap;

use serde::Serialize;

use alohomora::AlohomoraType;
use alohomora::context::Context;
use alohomora::policy::NoPolicy;
use alohomora::testing::TestContextData;
use alohomora_derive::{AlohomoraType, route, routes, RequestBBoxJson, ResponseBBoxJson};

// POST request data.
#[derive(RequestBBoxJson, ResponseBBoxJson, PartialEq, Debug)]
pub struct Nested {
    pub inner: alohomora::bbox::BBox<String, NoPolicy>,
    pub vec: Vec<alohomora::bbox::BBox<i64, NoPolicy>>,
}

#[derive(Serialize)]
pub struct Nested2 {
    pub f1: String,
    pub f2: f64,
}

#[derive(RequestBBoxJson, PartialEq, Debug)]
pub struct Simple {
    pub f1: alohomora::bbox::BBox<String, NoPolicy>,
    pub f2: Nested,
    pub f3: alohomora::bbox::BBox<u64, NoPolicy>,
    pub f4: HashMap<String, alohomora::bbox::BBox<u64, NoPolicy>>,
    pub f5: Option<alohomora::bbox::BBox<u64, NoPolicy>>,
}

#[derive(ResponseBBoxJson)]
#[response_bbox_json(as_is = [f3])]
pub struct Output {
    pub f1: alohomora::bbox::BBox<String, NoPolicy>,
    pub f2: Nested,
    pub f3: Nested2,
    pub f4: Option<Nested>,
    pub f5: String,
}

// Context derived from both request and also json post data.
#[derive(AlohomoraType)]
struct ContextData {
  // we acquire this from the post data via BBoxJson<Simple>.
  pub f1: alohomora::bbox::BBox<String, NoPolicy>,
  // we acquire this a cookie via BBoxRequest.
  pub cookie: alohomora::bbox::BBox<String, NoPolicy>,
}

// Notice that we need to include *BBoxForm<Simple>* (or BBoxJson<Simple>) in
// the trait generics, and NOT just Simple.
#[rocket::async_trait]
impl<'a, 'r> alohomora::rocket::FromBBoxRequestAndData<'a, 'r, alohomora::rocket::BBoxJson<Simple>> for ContextData {
    type BBoxError = ();
    async fn from_bbox_request_and_data(
        request: alohomora::rocket::BBoxRequest<'a, 'r>,
        form: &'_ alohomora::rocket::BBoxJson<Simple>,
    ) -> alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
        alohomora::rocket::BBoxRequestOutcome::Success(ContextData {
            f1: form.f1.clone(),
            cookie: request.cookies().get("mycookie").unwrap().value().to_owned(),
        })
    }
}

#[route(POST, "/", data = "<data>", with_data = "<context>")]
fn my_route(
    context: Context<ContextData>,
    data: alohomora::rocket::BBoxJson<Simple>,
) -> alohomora::rocket::JsonResponse<Output, TestContextData<()>> {
    let simple = Simple {
        f1: alohomora::bbox::BBox::new(String::from("hello"), NoPolicy {}),
        f2: Nested {
            inner: alohomora::bbox::BBox::new(String::from("bye"), NoPolicy {}),
            vec: vec![
                alohomora::bbox::BBox::new(-100, NoPolicy {}),
                alohomora::bbox::BBox::new(200, NoPolicy {})
            ],
        },
        f3: alohomora::bbox::BBox::new(55, NoPolicy {}),
        f4: HashMap::from([
            (String::from("k1"), alohomora::bbox::BBox::new(11, NoPolicy {})),
            (String::from("k2"), alohomora::bbox::BBox::new(12, NoPolicy {})),
        ]),
        f5: None,
    };

    // Check that input is parsed correctly.
    assert_eq!(data.into_inner(), simple);

    let output = Output {
        f1: alohomora::bbox::BBox::new(String::from("hi"), NoPolicy {}),
        f2: simple.f2,
        f3: Nested2 {
            f1: String::from("nestedf1"),
            f2: 22.5,
        },
        f4: None,
        f5: String::from("raw"),
    };

    // assert that context is constructed correctly.
    assert_eq!(context.data().unwrap().f1.as_ref().discard_box(), "hello");
    assert_eq!(context.data().unwrap().cookie.as_ref().discard_box(), "cookie value!");

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
        .cookie(rocket::http::Cookie::new("mycookie", "cookie value!"))
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
