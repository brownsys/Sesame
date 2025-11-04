extern crate sesame;
extern crate sesame_mysql;
extern crate sesame_rocket;

extern crate mysql;
extern crate rocket;
extern crate rocket_dyn_templates;
extern crate serde;

use sesame::pcon::PCon;
use sesame::policy::{AnyPolicy, NoPolicy};
use sesame::verified::{execute_verified, VerifiedRegion};
use sesame_rocket::rocket::{get, routes, PConRedirect, SesameRocket};

use rocket_dyn_templates::Template;
use std::sync::{Arc, Mutex};

mod backend;
mod config;
mod policy;
mod routes;

use backend::MySqlBackend;
use policy::context::YouChatContext;
use sesame::error::SesameResult;

// index will redirect to login, which will redirect to chat
#[get("/")]
fn index(context: YouChatContext) -> SesameResult<PConRedirect> {
    PConRedirect::to("/login", (), context)
}

// redirect login button request to the right page
#[get("/chat?<name>")]
pub(crate) fn to_chat(
    name: PCon<String, NoPolicy>,
    context: YouChatContext,
) -> SesameResult<PConRedirect> {
    // sanitize name
    let name_proc: PCon<String, AnyPolicy> = execute_verified(
        name,
        VerifiedRegion::new(|data: String| data.chars().filter(|c| c.is_alphabetic()).collect()),
    )
    .unwrap();

    PConRedirect::to("/chat/{}", (&name_proc,), context)
}

// builds server instance for use in testing
pub fn build_server() -> SesameRocket<rocket::Build> {
    // get config
    let config_path = "sample-config.toml";
    let config = config::parse(config_path).unwrap();

    // initialize backend
    let db_name = "chats";
    let backend = Arc::new(Mutex::new(
        MySqlBackend::new(
            &config.db_user,
            &config.db_password,
            &format!("{}", db_name),
            config.prime,
        )
        .unwrap(),
    ));

    // then make template
    let template_dir = config.template_dir.clone();
    let template = Template::try_custom(move |engines| {
        let result = engines
            .handlebars
            .register_templates_directory(".hbs", std::path::Path::new(&template_dir));
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    });

    // build and launch
    SesameRocket::build()
        .attach(template)
        .manage(backend)
        .manage(config)
        .mount("/", routes![index, to_chat])
        .mount("/login", routes![routes::login::login])
        .mount(
            "/chat",
            routes![
                routes::chat::show_chat,
                routes::chat::send,
                routes::groupchat::try_show_gc,
                routes::groupchat::send,
                routes::groupchat::try_delete
            ],
        )
        .mount(
            "/buggy",
            routes![routes::buggy::buggy_endpoint, routes::chat::send],
        )
}
