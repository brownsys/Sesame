use bbox::policy::{FrontendPolicy, Policy};
use bbox::rocket::BBoxRequest;
use bbox_derive::{route, routes, FromBBoxForm};
use std::any::Any;

pub struct TmpPolicy {}
impl Policy for TmpPolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &dyn Any) -> bool {
        true
    }
}
impl FrontendPolicy for TmpPolicy {
    fn from_request<'a, 'r>(_: &'a BBoxRequest<'a, 'r>) -> Self {
        TmpPolicy {}
    }
    fn from_cookie() -> Self {
        TmpPolicy {}
    }
}

// POST request data.
#[derive(FromBBoxForm)]
struct Simple {
    f1: bbox::bbox::BBox<String, TmpPolicy>,
    f3: bbox::bbox::BBox<u8, TmpPolicy>,
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
impl<'r> bbox::rocket::FromBBoxRequest<'r> for MyGuard {
    type BBoxError = &'static str;
    async fn from_bbox_request(
        _request: &'r bbox::rocket::BBoxRequest<'r, '_>,
    ) -> bbox::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
        let guard = MyGuard {
            value: String::from("ok"),
        };
        bbox::rocket::BBoxRequestOutcome::Success(guard)
    }
}

// Get request param.
#[derive(FromBBoxForm)]
struct Dog {
    name: bbox::bbox::BBox<String, TmpPolicy>,
    age: bbox::bbox::BBox<usize, TmpPolicy>,
}

// HTTP reuqest.
// POST /route/<num>?<dog>&<a>
// Example: /route/5?a=apple&dog.name=Max&dog.age=10
#[route(POST, "/route/<num>?<dog>&<a>", data = "<data>")]
fn my_route(
    guard: MyGuard,
    num: bbox::bbox::BBox<u8, TmpPolicy>,
    data: bbox::rocket::BBoxForm<Simple>,
    config: &rocket::State<Config>,
    a: bbox::bbox::BBox<String, TmpPolicy>,
    dog: Dog,
) -> bbox::rocket::BBoxRedirect {
    assert_eq!(guard.value, "ok");
    assert_eq!(config.debug_mode, false);
    assert_eq!(config.admins.len(), 1);
    assert!(config.admins.contains("test@email.com"));
    assert_eq!(data.f1.temporary_unbox(), "str1");
    assert_eq!(*data.f3.temporary_unbox(), 10);
    assert_eq!(*num.temporary_unbox(), 5);

    // all good.
    bbox::rocket::BBoxRedirect::to("ok", vec![])
}

#[test]
fn simple_from_bbox_form_test() {
    let rocket = bbox::rocket::BBoxRocket::<::rocket::Build>::build()
        .manage(Config::new("test@email.com"))
        .mount("/test", routes![my_route]);
}
