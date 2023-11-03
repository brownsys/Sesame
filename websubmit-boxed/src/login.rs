use std::collections::HashMap;
use rocket::State;

use bbox::context::Context;
use bbox::policy::{NoPolicy, AnyPolicy}; //{AnyPolicy, NoPolicy, PolicyAnd, SchemaPolicy};
use bbox::rocket::BBoxTemplate;
use bbox_derive::get;

use crate::apikey::ApiKey;
use crate::config::Config;
use crate::policies::ContextData;

#[get("/")]
pub(crate) fn login(config: &State<Config>, context: Context<ApiKey, ContextData>,) -> BBoxTemplate {
    let mut ctx = HashMap::new();
    ctx.insert("CLASS_ID", config.class.clone());
    ctx.insert("parent", String::from("layout"));
    BBoxTemplate::render("login", &ctx, &context)
}
