#![feature(build_hasher_simple_hash_one)]
extern crate serde;
extern crate mysql;
extern crate rocket;
extern crate rocket_dyn_templates;

use std::sync::{Mutex, Arc};
use rocket_dyn_templates::Template;
use alohomora::bbox::BBox; 
use alohomora::policy::{NoPolicy, AnyPolicy};
use alohomora::pure::{PrivacyPureRegion, execute_pure};
use alohomora::rocket::{BBoxRocket, BBoxRedirect};
use alohomora_derive::{get, routes};
use backend::MySqlBackend;
use context::YouChatContext;

mod backend;
mod config;
mod login;
mod chat;
mod buggy;
mod groupchat;
mod common;
mod policies;
mod context;

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
        PrivacyPureRegion::new(|data: String|
            data.chars().filter(|c| c.is_alphabetic()).collect()
        )
    ).unwrap();

    BBoxRedirect::to("/chat/{}", (&name_proc,), context)
}

use std::sync::Once;
static INIT: Once = Once::new();

// Register all policies. #[schema_policy(...)] does not work on mac.
pub fn register_policies() {
    // Call this once and ONLY once.
    INIT.call_once(|| {
        use policies::ChatAccessPolicy;
        alohomora::policy::add_schema_policy::<ChatAccessPolicy>(String::from("chats"), 0);
        alohomora::policy::add_schema_policy::<ChatAccessPolicy>(String::from("chats"), 1);
        alohomora::policy::add_schema_policy::<ChatAccessPolicy>(String::from("chats"), 2);
        alohomora::policy::add_schema_policy::<ChatAccessPolicy>(String::from("chats"), 4);
    });
}

// builds server instance for use in testing
pub fn build_server() -> BBoxRocket<rocket::Build> {
    register_policies();

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
            config.prime
        ).unwrap()
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
        .mount("/chat", routes![chat::show_chat, chat::send, 
                                groupchat::try_show_gc, groupchat::send, groupchat::try_delete])
        .mount("/buggy", routes![buggy::buggy_endpoint, chat::send])
}
