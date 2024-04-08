extern crate alohomora;
extern crate clap;
extern crate lettre;
extern crate lettre_email;
extern crate mysql;
extern crate rocket;
#[macro_use]
extern crate slog;
extern crate serde_derive;
extern crate slog_term;

use alohomora::policy::NoPolicy;
use alohomora::rocket::{BBoxCookieJar, BBoxRedirect, BBoxRocket, BBoxRoute,
                        get, routes};
use alohomora::context::Context;
use backend::MySqlBackend;
use rocket::fs::FileServer;
use rocket::State;
use rocket_dyn_templates::Template;
use std::sync::{Arc, Mutex};


mod admin;
mod apikey;
mod args;
mod backend;
mod config;
mod email;
/*
mod grades;*/
mod helpers;
mod login;/*
mod manage;
*/
mod policies;
/*
mod predict;*/
mod questions;


pub fn new_logger() -> slog::Logger {
    use slog::Drain;
    use slog::Logger;
    use slog_term::term_full;
    Logger::root(Mutex::new(term_full()).fuse(), o!())
}


#[get("/")]
fn index(cookies: BBoxCookieJar<'_, '_>, backend: &State<Arc<Mutex<MySqlBackend>>>, context: Context<policies::ContextData>) -> BBoxRedirect {
    if let Some(cookie) = cookies.get::<NoPolicy>("apikey") {
        let apikey = cookie.into();
        match apikey::check_api_key(&*backend, &apikey, context) {
            Ok(_user) => BBoxRedirect::to2("/leclist"),
            Err(_) => BBoxRedirect::to2("/login"),
        }
    } else {
        BBoxRedirect::to2("/login")
    }
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
            .register_templates_directory(".hbs", &template_dir);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    });

    
    if let Err(e) = BBoxRocket::build()
        .attach(template)
        .manage(backend)
        .manage(config)
        .mount(
            "/css",
            BBoxRoute::from(FileServer::from(format!("{}/css", resource_dir))),
        )
        .mount(
            "/js",
            BBoxRoute::from(FileServer::from(format!("{}/js", resource_dir))),
        )
        .mount("/", routes![index])
        .mount(
            "/questions",
            routes![questions::questions, questions::questions_submit],
        )
        .mount("/apikey/check", routes![apikey::check])
        
        .mount("/apikey/generate", routes![apikey::generate])/* 
        .mount(
            "/grades",
            routes![grades::grades, grades::editg, grades::editg_submit],
        )*/
        .mount("/answers", routes![questions::composed_answers])
        .mount("/leclist", routes![questions::leclist])/*
        .mount(
            "/predict",
            routes![predict::predict, predict::predict_grade],
        )
        */
        .mount("/login", routes![login::login])
        .mount(
            "/admin/lec/add",
            routes![admin::lec_add, admin::lec_add_submit],
        )
        .mount("/admin/users", routes![admin::get_registered_users])
        .mount(
            "/admin/lec",
            routes![admin::lec, admin::addq, admin::editq, admin::editq_submit],
        )/*
        .mount("/manage/users", routes![manage::get_aggregate_grades])
        */
        .launch()
        .await
    {
        println!("Whoops, didn't launch!");
        drop(e);
    };
    
}
