use rocket::fs::FileServer;
use rocket::Build;
use rocket::{routes, Rocket};
use rocket_dyn_templates::Template;
use slog::o;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod admin;
mod apikey;
mod args;
mod backend;
mod config;
mod email;
mod grades;
mod helpers;
mod index;
mod login;
mod manage;
mod predict;
mod questions;

pub use args::parse_args;

fn new_logger() -> slog::Logger {
    use slog::Drain;
    use slog::Logger;
    use slog_term::term_full;
    Logger::root(Mutex::new(term_full()).fuse(), o!())
}

pub fn make_rocket(args: args::Args) -> Rocket<Build> {
    let config = args.config;

    let backend = Arc::new(Mutex::new(
        backend::MySqlBackend::new(
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
    let template = Template::custom(move |engines| {
        engines
            .handlebars
            .register_templates_directory(".hbs", Path::new(&template_dir))
            .expect("Failed to set template path!");
    });

    Rocket::build()
        .attach(template)
        .manage(backend)
        .manage(config)
        .mount("/css", FileServer::from(format!("{}/css", resource_dir)))
        .mount("/js", FileServer::from(format!("{}/js", resource_dir)))
        .mount("/", routes![index::index])
        .mount(
            "/questions",
            routes![questions::questions, questions::questions_submit],
        )
        .mount("/apikey/check", routes![apikey::check])
        .mount("/apikey/generate", routes![apikey::generate])
        .mount(
            "/grades",
            routes![grades::grades, grades::editg, grades::editg_submit],
        )
        .mount("/answers", routes![questions::answers, questions::answers_for_discussion_leaders])
        .mount("/leclist", routes![questions::leclist])
        .mount(
            "/predict",
            routes![predict::predict, predict::predict_grade, predict::retrain_model],
        )
        .mount("/login", routes![login::login])
        .mount(
            "/admin/lec/add",
            routes![admin::lec_add, admin::lec_add_submit],
        )
        .mount("/admin/users", routes![admin::get_registered_users])
        .mount(
            "/admin/lec",
            routes![admin::lec, admin::addq, admin::editq, admin::editq_submit],
        )
        .mount(
            "/manage",
            routes![
                manage::get_aggregate_gender,
                manage::get_aggregate_remote,
                manage::get_aggregate_remote_buggy,
                manage::get_list_for_employers,
                manage::get_list_for_employers_buggy
            ],
        )
}
