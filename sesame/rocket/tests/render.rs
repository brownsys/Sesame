use std::env;

use rocket::http::Status;
use rocket_dyn_templates::Template;
use sesame_rocket::rocket::SesameRocket;
use sesame_rocket::test_route;
use sesame_rocket::testing::SesameClient;

use crate::render_application::routes::{
    render_newtype, render_struct_variant, render_tuple, render_tuple_struct, render_unit,
};

mod render_application;

#[test]
fn test_render_end_to_end() {
    let template = Template::try_custom(move |engines| {
        let result = engines
            .handlebars
            .register_templates_directory(".hbs", "tests/render_application");
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    });

    env::set_var("ROCKET_template_dir", "tests/render_application");
    let rocket = SesameRocket::build()
        .attach(template)
        .mount(
            "/",
            vec![
                test_route!(Get, "/tuple_struct", render_tuple_struct),
                test_route!(Get, "/unit", render_unit),
                test_route!(Get, "/newtype", render_newtype),
                test_route!(Get, "/tuple", render_tuple),
                test_route!(Get, "/struct_variant", render_struct_variant),
            ],
        );

    let client = SesameClient::tracked(rocket).expect("valid `Rocket`");

    // Tuple struct: Point(5, 10) renders as an array.
    let response = client.get("/tuple_struct").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("5 10 "));

    // Unit variant: Shape::Unknown renders as the variant name string.
    let response = client.get("/unit").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("Unknown"));

    // Newtype variant: Shape::Circle(7) renders as {"Circle": 7}.
    let response = client.get("/newtype").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("7"));

    // Tuple variant: Shape::Rectangle(3, 4) renders as {"Rectangle": [3, 4]}.
    let response = client.get("/tuple").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("3 4 "));

    // Struct variant: Shape::Named { width: 6, label: "hello" } renders as {"Named": {fields}}.
    let response = client.get("/struct_variant").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("6,hello"));
}
