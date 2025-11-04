use crate::policy::context::YouChatContext;
use sesame_rocket::error::SesameRenderResult;
use sesame_rocket::render::PConRender;
use sesame_rocket::rocket::{get, PConTemplate};

#[derive(PConRender)]
struct Empty {}

#[get("/")]
pub(crate) fn login(context: YouChatContext) -> SesameRenderResult<PConTemplate> {
    let ctx = Empty {};
    PConTemplate::render("login", &ctx, context)
}
