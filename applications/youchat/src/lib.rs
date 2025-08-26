extern crate mysql;
extern crate rocket;
extern crate rocket_dyn_templates;
extern crate serde;

use alohomora::bbox::BBox;
use alohomora::policy::{AnyPolicy, NoPolicy};
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use alohomora::rocket::{get, routes, BBoxRedirect, BBoxRocket};
use rocket_dyn_templates::Template;
use std::sync::{Arc, Mutex};

mod backend;
mod buggy;
mod chat;
mod common;
mod config;
mod context;
mod groupchat;
mod login;
mod policies;

use backend::MySqlBackend;
use context::YouChatContext;

// index will redirect to login, which will redirect to chat
#[get("/")]
fn index(context: YouChatContext) -> BBoxRedirect {
    BBoxRedirect::to("/login", (), context)
}

// redirect login button request to the right page
#[get("/chat?<name>")]
pub(crate) fn to_chat(name: BBox<String, NoPolicy>, context: YouChatContext) -> BBoxRedirect {
    // sanitize name
    let name_proc: BBox<String, AnyPolicy> = execute_pure(
        name,
        PrivacyPureRegion::new(|data: String| data.chars().filter(|c| c.is_alphabetic()).collect()),
    )
    .unwrap();

    BBoxRedirect::to("/chat/{}", (&name_proc,), context)
}

// builds server instance for use in testing
pub fn build_server() -> BBoxRocket<rocket::Build> {
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
    BBoxRocket::build()
        .attach(template)
        .manage(backend)
        .manage(config)
        .mount("/", routes![index, to_chat])
        .mount("/login", routes![login::login])
        .mount(
            "/chat",
            routes![
                chat::show_chat,
                chat::send,
                groupchat::try_show_gc,
                groupchat::send,
                groupchat::try_delete
            ],
        )
        .mount("/buggy", routes![buggy::buggy_endpoint, chat::send])
}
