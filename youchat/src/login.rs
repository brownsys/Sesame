use alohomora::rocket::BBoxTemplate;
use alohomora_derive::{get, BBoxRender};
use crate::context::YouChatContext;

#[derive(BBoxRender)]
struct Empty {}

#[get("/")]
pub(crate) fn login(context: YouChatContext) -> BBoxTemplate {
    let ctx = Empty{};
    BBoxTemplate::render("login", &ctx, context)
}