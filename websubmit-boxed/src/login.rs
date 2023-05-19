use std::collections::HashMap;

use rocket::State;
use rocket_dyn_templates::Template;

use crate::config::Config;

#[get("/")]
pub(crate) fn login(config: &State<Config>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("CLASS_ID", config.class.clone());
    ctx.insert("parent", String::from("layout"));
    Template::render("login", &ctx)
}
