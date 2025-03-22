use alohomora::bbox::BBoxRender;
use alohomora::rocket::{get, post, BBoxTemplate};
use crate::context::YouChatContext;

#[derive(BBoxRender)]
struct Empty {}

#[get("/")]
pub(crate) fn login(context: YouChatContext) -> BBoxTemplate {
    let ctx = Empty{};
    BBoxTemplate::render("login", &ctx, context)
}
