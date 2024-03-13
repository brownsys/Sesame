use alohomora::context::Context;
use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy};
use alohomora_derive::{route, routes, FromBBoxForm};

use std::any::Any;
use rocket::http::Cookie;
use rocket::Request;
use alohomora::pcr::PrivacyCriticalRegion;
use alohomora::unbox::unbox;

#[derive(Clone)]
pub struct TmpPolicy {}
impl Policy for TmpPolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &dyn Any) -> bool {
        true
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl FrontendPolicy for TmpPolicy {
    fn from_request(_: &'_ Request<'_>) -> Self {
        TmpPolicy {}
    }
    fn from_cookie<'a, 'r>(_name: &str, _cookie: &'a Cookie<'static>, _request: &'a Request<'r>) -> Self {
        TmpPolicy {}
    }
}

// POST request data.
#[derive(FromBBoxForm)]
struct Simple {
    f1: alohomora::bbox::BBox<String, TmpPolicy>,
    f3: alohomora::bbox::BBox<u8, TmpPolicy>,
}

// This struct serves as a request guard.
struct Config {
    pub debug_mode: bool,
    pub admins: std::collections::HashSet<String>,
}
impl Config {
    pub fn new(admin: &str) -> Self {
        let mut c = Config {
            debug_mode: false,
            admins: std::collections::HashSet::new(),
        };
        c.admins.insert(String::from(admin));
        c
    }
}

// Another request guard.
struct MyGuard {
    pub value: String,
}
#[rocket::async_trait]
impl<'a, 'r> alohomora::rocket::FromBBoxRequest<'a, 'r> for MyGuard {
    type BBoxError = &'static str;
    async fn from_bbox_request(
        _request: alohomora::rocket::BBoxRequest<'a, 'r>,
    ) -> alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
        let guard = MyGuard {
            value: String::from("ok"),
        };
        alohomora::rocket::BBoxRequestOutcome::Success(guard)
    }
}

// Get request param.
#[derive(FromBBoxForm)]
struct Dog {
    name: alohomora::bbox::BBox<String, TmpPolicy>,
    age: alohomora::bbox::BBox<usize, TmpPolicy>,
}

// TODO(babman): get endpoint
// TODO(babman): actually invoke endpoint
// HTTP request.
// POST /route/<num>?<dog>&<a>
// Example: /route/5?a=apple&dog.name=Max&dog.age=10
#[route(POST, "/route/<num>?<dog>&<a>", data = "<data>")]
fn my_route(
    guard: MyGuard,
    num: alohomora::bbox::BBox<u8, TmpPolicy>,
    data: alohomora::rocket::BBoxForm<Simple>,
    config: &rocket::State<Config>,
    a: alohomora::bbox::BBox<String, TmpPolicy>,
    dog: Dog,
) -> alohomora::rocket::BBoxRedirect {
    assert_eq!(guard.value, "ok");
    assert_eq!(config.debug_mode, false);
    assert_eq!(config.admins.len(), 1);
    assert!(config.admins.contains("test@email.com"));

    let context = Context::new(Option::None::<()>, String::from(""), ());
    let result = unbox(
        (num.clone(), a.clone(), data.f1.clone(), data.f3.clone(), dog.name, dog.age),
        &context,
        PrivacyCriticalRegion::new(|(num, a, f1, f3, name, age), _| {
            assert_eq!(&f1, "str1");
            assert_eq!(f3, 10);
            assert_eq!(num, 5);
            assert_eq!(&a, "apple");
            assert_eq!(&name, "Max");
            assert_eq!(age, 10);
        }),
        ());
    result.unwrap();

    // all good.
    alohomora::rocket::BBoxRedirect::to("/page/{}/{}/{}/{}", (&a, &num, &"test", &10))
}

#[test]
fn simple_from_bbox_form_test() {
    let _rocket = alohomora::rocket::BBoxRocket::<::rocket::Build>::build()
        .manage(Config::new("test@email.com"))
        .mount("/test", routes![my_route]);
}
