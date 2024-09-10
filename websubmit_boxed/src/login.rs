use rocket::State;
use std::collections::HashMap;

use alohomora::context::Context;
use alohomora::rocket::{get, BBoxTemplate};

use crate::config::Config;
use crate::policies::ContextData;

#[get("/")]
pub(crate) fn login(config: &State<Config>, context: Context<ContextData>) -> BBoxTemplate {
    let mut ctx = HashMap::new();
    ctx.insert("CLASS_ID", config.class.clone());
    ctx.insert("parent", String::from("layout"));
    BBoxTemplate::render("login", &ctx, context)
}
