use bbox_derive::{route, FromBBoxForm, routes};

// POST request data.
#[derive(FromBBoxForm)]
struct Simple {
    f1: bbox::bbox::BBox<String>,
    f3: bbox::bbox::BBox<u8>,
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
    name: bbox::bbox::BBox<String>,
    age: bbox::bbox::BBox<usize>,
}

// HTTP reuqest.
// POST /route/<num>?<dog>&<a>
// Example: /route/5?a=apple&dog.name=Max&dog.age=10
#[route(POST, "/route/<num>?<dog>&<a>", data = "<data>")]
fn my_route(
    guard: MyGuard,
    num: bbox::bbox::BBox<u8>,
    data: bbox::rocket::BBoxForm<Simple>,
    config: &rocket::State<Config>,
    a: bbox::bbox::BBox<String>,
    dog: Dog,
) -> bbox::rocket::BBoxRedirect {
    assert_eq!(guard.value, "ok");
    assert_eq!(config.debug_mode, false);
    assert_eq!(config.admins.len(), 1);
    assert!(config.admins.contains("test@email.com"));
    assert_eq!(data.f1.test_unbox(), "str1");
    assert_eq!(*data.f3.test_unbox(), 10);
    assert_eq!(*num.test_unbox(), 5);

    // all good.
    bbox::rocket::BBoxRedirect::to("ok", vec![])
}

#[test]
fn simple_from_bbox_form_test() {
    let rocket = bbox::rocket::BBoxRocket::<::rocket::Build>::build()
        .manage(Config::new("test@email.com"))
        .mount("/test", routes![my_route]);
}
