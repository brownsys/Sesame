extern crate clap;
extern crate mysql;
#[macro_use]
extern crate rocket;
extern crate lettre;
extern crate lettre_email;
#[macro_use]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate serde_derive;

mod admin;
mod apikey;
mod args;
mod backend;
mod config;
mod email;
mod grades;
// mod helpers;
mod login;
// mod manage;
// mod predict;
mod index;
mod questions;

use backend::MySqlBackend;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use std::sync::{Arc, Mutex};

pub fn new_logger() -> slog::Logger {
    use slog::Drain;
    use slog::Logger;
    use slog_term::term_full;
    Logger::root(Mutex::new(term_full()).fuse(), o!())
}

#[rocket::main]
async fn main() {
    let args = args::parse_args();
    let config = args.config;

    let backend = Arc::new(Mutex::new(
        MySqlBackend::new(
            &config.db_user,
            &config.db_password,
            &format!("{}", args.class),
            Some(new_logger()),
            config.prime,
        )
        .unwrap(),
    ));

    let template_dir = config.template_dir.clone();
    let resource_dir = config.resource_dir.clone();

    let template = Template::try_custom(move |engines| {
        let result = engines
            .handlebars
            .register_templates_directory(".hbs", std::path::Path::new(&template_dir));
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    });

    if let Err(e) = rocket::build()
        .attach(template)
        .manage(backend)
        .manage(config)
        .mount(
            "/css", 
            FileServer::from(format!("{}/css", resource_dir)))
        .mount(
            "/js", 
            FileServer::from(format!("{}/js", resource_dir)))
        .mount(
            "/", 
            routes![index::index])
        .mount(
            "/questions",
            routes![questions::questions, questions::questions_submit],
        )
        .mount(
            "/apikey/check", 
            routes![apikey::check])
        .mount(
            "/apikey/generate", 
            routes![apikey::generate])
        .mount(
            "/answers", 
            routes![questions::answers])
        .mount(
            "/leclist", 
            routes![questions::leclist])
        .mount(
            "/login", 
            routes![login::login])
        .mount(
            "/admin/lec/add",
            routes![admin::lec_add, admin::lec_add_submit],
        )
        .mount(
            "/admin/users", 
            routes![admin::get_registered_users])
        .mount(
            "/admin/lec",
            routes![admin::lec, admin::addq, admin::editq, admin::editq_submit],
        )
        .launch()
        .await
    {
        println!("Whoops, didn't launch!");
        drop(e);
    };
}
