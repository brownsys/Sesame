use std::collections::HashMap;

use serde::Serialize;

use sesame::context::Context;
use sesame::policy::NoPolicy;
use sesame::testing::TestContextData;
use sesame_derive::{route, routes, RequestPConJson, ResponsePConJson, SesameType};

// POST request data.
#[derive(RequestPConJson, ResponsePConJson, PartialEq, Debug)]
pub struct Nested {
    pub inner: sesame::pcon::PCon<String, NoPolicy>,
    pub vec: Vec<sesame::pcon::PCon<i64, NoPolicy>>,
}

#[derive(Serialize)]
pub struct Nested2 {
    pub f1: String,
    pub f2: f64,
}

#[derive(RequestPConJson, PartialEq, Debug)]
pub struct Simple {
    pub f1: sesame::pcon::PCon<String, NoPolicy>,
    pub f2: Nested,
    pub f3: sesame::pcon::PCon<u64, NoPolicy>,
    pub f4: HashMap<String, sesame::pcon::PCon<u64, NoPolicy>>,
    pub f5: Option<sesame::pcon::PCon<u64, NoPolicy>>,
}

#[derive(ResponsePConJson)]
#[response_pcon_json(as_is = [f3])]
pub struct Output {
    pub f1: sesame::pcon::PCon<String, NoPolicy>,
    pub f2: Nested,
    pub f3: Nested2,
    pub f4: Option<Nested>,
    pub f5: String,
}

// Context derived from both request and also json post data.
#[derive(SesameType)]
struct ContextData {
    // we acquire this from the post data via PConJson<Simple>.
    pub f1: sesame::pcon::PCon<String, NoPolicy>,
    // we acquire this a cookie via PConRequest.
    pub cookie: sesame::pcon::PCon<String, NoPolicy>,
}

// Notice that we need to include *PConForm<Simple>* (or PConJson<Simple>) in
// the trait generics, and NOT just Simple.
#[rocket::async_trait]
impl<'a, 'r>
    sesame_rocket::rocket::FromPConRequestAndData<'a, 'r, sesame_rocket::rocket::PConJson<Simple>>
    for ContextData
{
    type PConError = ();
    async fn from_pcon_request_and_data(
        request: sesame_rocket::rocket::PConRequest<'a, 'r>,
        form: &'_ sesame_rocket::rocket::PConJson<Simple>,
    ) -> sesame_rocket::rocket::PConRequestOutcome<Self, Self::PConError> {
        sesame_rocket::rocket::PConRequestOutcome::Success(ContextData {
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

#[route(POST, "/", data = "<data>", with_data = "<context>")]
fn my_route(
    context: Context<ContextData>,
    data: sesame_rocket::rocket::PConJson<Simple>,
) -> sesame_rocket::rocket::JsonResponse<Output, TestContextData<()>> {
    let simple = Simple {
        f1: sesame::pcon::PCon::new(String::from("hello"), NoPolicy {}),
        f2: Nested {
            inner: sesame::pcon::PCon::new(String::from("bye"), NoPolicy {}),
            vec: vec![
                sesame::pcon::PCon::new(-100, NoPolicy {}),
                sesame::pcon::PCon::new(200, NoPolicy {}),
            ],
        },
        f3: sesame::pcon::PCon::new(55, NoPolicy {}),
        f4: HashMap::from([
            (String::from("k1"), sesame::pcon::PCon::new(11, NoPolicy {})),
            (String::from("k2"), sesame::pcon::PCon::new(12, NoPolicy {})),
        ]),
        f5: None,
    };

    // Check that input is parsed correctly.
    assert_eq!(data.into_inner(), simple);

    let output = Output {
        f1: sesame::pcon::PCon::new(String::from("hi"), NoPolicy {}),
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
    assert_eq!(
        context.data().unwrap().cookie.as_ref().discard_box(),
        "cookie value!"
    );

    println!("test");
    // Return result.
    sesame_rocket::rocket::JsonResponse::from((output, Context::test(())))
}

#[test]
fn simple_from_pcon_form_test() {
    let rocket = sesame_rocket::rocket::SesameRocket::<::rocket::Build>::build()
        .mount("/", routes![my_route]);

    // Create a client.
    let client = sesame_rocket::testing::SesameClient::tracked(rocket).expect("valid `Rocket`");
    let response = client
        .post("/")
        .cookie(rocket::http::Cookie::new("mycookie", "cookie value!"))
        .header(rocket::http::ContentType::JSON)
        .body(
            "{\
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
        }",
        )
        .dispatch();

    // Validate response.
    use serde_json::{Map, Number, Value};
    use std::iter::FromIterator;
    assert_eq!(response.status(), rocket::http::Status::new(200));

    let response: Value = response.into_json().unwrap();
    assert_eq!(
        response,
        Value::Object(Map::from_iter([
            (String::from("f1"), Value::String(String::from("hi"))),
            (
                String::from("f2"),
                Value::Object(Map::from_iter([
                    (String::from("inner"), Value::String(String::from("bye"))),
                    (
                        String::from("vec"),
                        Value::Array(vec![
                            Value::Number(Number::from(-100i64)),
                            Value::Number(Number::from(200i64)),
                        ])
                    ),
                ]))
            ),
            (
                String::from("f3"),
                Value::Object(Map::from_iter([
                    (String::from("f1"), Value::String(String::from("nestedf1"))),
                    (
                        String::from("f2"),
                        Value::Number(Number::from_f64(22.5f64).unwrap())
                    ),
                ]))
            ),
            (String::from("f4"), Value::Null),
            (String::from("f5"), Value::String(String::from("raw"))),
        ]))
    );
}
