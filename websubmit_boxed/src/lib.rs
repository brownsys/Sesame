use alohomora::policy::add_schema_policy;
use alohomora::rocket::{routes, BBoxRocket, BBoxRoute};
use rocket::fs::FileServer;
use rocket::Build;
use rocket_dyn_templates::Template;
use slog::o;
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
mod policies;
mod predict;
mod questions;

pub use args::parse_args;

fn new_logger() -> slog::Logger {
    use slog::Drain;
    use slog::Logger;
    use slog_term::term_full;
    Logger::root(Mutex::new(term_full()).fuse(), o!())
}

pub fn make_rocket(args: args::Args) -> BBoxRocket<Build> {
    // println!("args riocets are {:?}", args);
    add_schema_policy::<policies::AnswerAccessPolicy>(String::from("answers"), 0usize);
    add_schema_policy::<policies::AnswerAccessPolicy>(String::from("answers"), 1usize);
    add_schema_policy::<policies::AnswerAccessPolicy>(String::from("answers"), 2usize);
    add_schema_policy::<policies::AnswerAccessPolicy>(String::from("answers"), 3usize);
    add_schema_policy::<policies::AnswerAccessPolicy>(String::from("answers"), 4usize);
    add_schema_policy::<policies::AnswerAccessPolicy>(String::from("answers"), 5usize);

    add_schema_policy::<policies::UserProfilePolicy>(String::from("users"), 5usize);
    add_schema_policy::<policies::UserProfilePolicy>(String::from("users"), 6usize);
    add_schema_policy::<policies::UserProfilePolicy>(String::from("users"), 7usize);

    add_schema_policy::<policies::KAnonymityPolicy>(String::from("agg_gender"), 1usize);
    add_schema_policy::<policies::KAnonymityPolicy>(String::from("agg_remote"), 1usize);

    add_schema_policy::<policies::AggregateAccessPolicy>(String::from("agg_gender"), 1usize);
    add_schema_policy::<policies::AggregateAccessPolicy>(String::from("agg_remote"), 1usize);

    add_schema_policy::<policies::MLTrainingPolicy>(String::from("ml_training"), 0usize);
    add_schema_policy::<policies::MLTrainingPolicy>(String::from("ml_training"), 1usize);

    add_schema_policy::<policies::EmployersReleasePolicy>(
        String::from("employers_release"),
        0usize,
    );
    add_schema_policy::<policies::EmployersReleasePolicy>(
        String::from("employers_release"),
        1usize,
    );

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
    let template = Template::try_custom(move |engines| {
        let result = engines
            .handlebars
            .register_templates_directory(".hbs", &template_dir);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    });

    BBoxRocket::build()
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
        .mount("/answers", routes![
            questions::composed_answers, 
            questions::naive_answers, 
            questions::answers_for_discussion_leaders,
            questions::answers_for_discussion_leaders_naive,
        ])
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
