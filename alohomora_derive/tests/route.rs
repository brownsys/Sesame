use alohomora::context::Context;
use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy};
use alohomora::rocket::BBoxRequest;
use alohomora_derive::{route, routes, FromBBoxForm};

use std::any::Any;

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
impl<'r> alohomora::rocket::FromBBoxRequest<'r> for MyGuard {
    type BBoxError = &'static str;
    async fn from_bbox_request(
        _request: &'r alohomora::rocket::BBoxRequest<'r, '_>,
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
    let context = Context::new(Option::None::<()>, String::from(""), ());

    assert_eq!(guard.value, "ok");
    assert_eq!(config.debug_mode, false);
    assert_eq!(config.admins.len(), 1);
    assert!(config.admins.contains("test@email.com"));
    assert_eq!(data.f1.unbox(&context), "str1");
    assert_eq!(*data.f3.unbox(&context), 10);
    assert_eq!(*num.unbox(&context), 5);
    assert_eq!(a.unbox(&context), "apple");
    assert_eq!(dog.name.unbox(&context), "Max");
    assert_eq!(*dog.age.unbox(&context), 10);

    // all good.
    alohomora::rocket::BBoxRedirect::to("/page/{}/{}/{}/{}", (&a, &num, &"test", &10))
}

#[test]
fn simple_from_bbox_form_test() {
    let _rocket = alohomora::rocket::BBoxRocket::<::rocket::Build>::build()
        .manage(Config::new("test@email.com"))
        .mount("/test", routes![my_route]);
}
