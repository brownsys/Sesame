use rocket::State;
use std::collections::HashMap;

use alohomora::rocket::{get, BBoxTemplate};

use crate::config::Config;
use crate::policies::Context;

#[get("/")]
pub(crate) fn login(config: &State<Config>, context: Context) -> BBoxTemplate {
    let mut ctx = HashMap::new();
    ctx.insert("CLASS_ID", config.class.clone());
    ctx.insert("parent", String::from("layout"));
    BBoxTemplate::render("login", &ctx, context)
}
