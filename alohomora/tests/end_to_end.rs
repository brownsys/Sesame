use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use rocket::http::{ContentType, Status};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_dyn_templates::Template;
use alohomora::rocket::BBoxRocket;
use alohomora::test_route;
use alohomora::testing::BBoxClient;
use crate::application::db::DB;
use crate::application::routes::{login, post_grade, read_all_grades, read_grades};

const ALL_GRADES: &'static str = "<html>
  <body>

        <tr>
          <td>1</td>
          <td>kinan</td>
          <td>90</td>
        </tr>

        <tr>
          <td>2</td>
          <td>kinan</td>
          <td>80</td>
        </tr>

        <tr>
          <td>3</td>
          <td>artem</td>
          <td>100</td>
        </tr>

  </body>
</html>";

const KINAN_GRADES: &'static str = "<html>
  <body>

        <tr>
          <td>1</td>
          <td>kinan</td>
          <td>90</td>
        </tr>

        <tr>
          <td>2</td>
          <td>kinan</td>
          <td>80</td>
        </tr>

  </body>
</html>";

mod application;

#[test]
fn test_end_to_end_application() {
    let mut db = DB::connect();
    db.prime();

    let template = Template::try_custom(move |engines| {
        let result = engines
            .handlebars
            .register_templates_directory(".hbs", "tests/application");
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    });

    // Test setting up cores.
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            ["Get", "Post", "Put", "Delete", "Options"]
                .iter()
                .map(|s| FromStr::from_str(s).unwrap())
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()
        .expect("Failed to setup cors configuration.");

    // Create a rocket instance and mount route.
    env::set_var("ROCKET_template_dir", "tests/application");
    let rocket = BBoxRocket::build()
        .attach(template)
        .manage(Arc::new(Mutex::new(db)))
        .attach(cors.clone())
        .mount("/", alohomora::rocket::catch_all_options_routes())
        .mount("/", vec![
            test_route!(Get, "/login/<user>", login),
            test_route!(Post, "/submit", post_grade),
            test_route!(Get, "/read_grades", read_grades),
            test_route!(Get, "/all", read_all_grades),
        ]);

    // Create a client.
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");

    // First, log in as admin to write some grades.
    let response = client.get("/login/admin").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("success"));

    // Write some grades.
    let response = client.post("/submit")
        .header(ContentType::Form)
        .body("0=kinan&1=90")
        .dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("success"));

    let response = client.post("/submit")
        .header(ContentType::Form)
        .body("0=kinan&1=80")
        .dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("success"));

    let response = client.post("/submit")
        .header(ContentType::Form)
        .body("0=artem&1=100")
        .dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("success"));

    // Admin can view all grades.
    let response = client.get("/all").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), ALL_GRADES);

    // Log in as Kinan to view my grades.
    let response = client.get("/login/kinan").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), String::from("success"));

    let response = client.get("/read_grades").dispatch();
    assert_eq!(response.status(), Status::new(200));
    assert_eq!(response.into_string().unwrap(), KINAN_GRADES);

    // Cannot view all grades as Kinan, cannot post grades.
    let response = client.get("/all").dispatch();
    assert_eq!(response.status(), Status::new(500));

    let response = client.post("/submit")
        .header(ContentType::Form)
        .body("0=kinan&1=100")
        .dispatch();
    assert_eq!(response.status(), Status::new(500));
}