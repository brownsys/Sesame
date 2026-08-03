// The two servers for the end-to-end experiment, kept apart from the payload
// modules so the benchmark data stays independent of how it is served.
//
// Each server exposes the same two endpoints, one per payload. The handlers do
// nothing but look up a payload that `common` prebuilt during setup and hand it
// to the matching render helper -- no database, no computation, nothing that
// would show up in the measurement other than the render itself.

use std::path::Path;

use rocket_dyn_templates::Template;

use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_rocket::error::SesameRenderResult;
use sesame_rocket::rocket::{get, routes, PConTemplate, SesameRocket};

use crate::common::{chat, simple};

// --- Sesame routes. ---

#[get("/bench/simple")]
fn sesame_simple(context: Context<()>) -> SesameRenderResult<PConTemplate> {
    simple::render_sesame(simple::sesame(), context)
}

// Sesame routes receive path parameters already wrapped, so this handler pays
// one `discard_box` that the plain handler below does not. It is a move with no
// policy check attached, but it is not literally nothing.
#[get("/bench/chat/<n>")]
fn sesame_chat(
    n: PCon<usize, NoPolicy>,
    context: Context<()>,
) -> SesameRenderResult<PConTemplate> {
    chat::render_sesame(chat::sesame(n.discard_box()), context)
}

// --- Plain Rocket routes. ---

#[rocket::get("/bench/simple")]
fn plain_simple() -> Template {
    simple::render_plain(simple::plain())
}

#[rocket::get("/bench/chat/<n>")]
fn plain_chat(n: usize) -> Template {
    chat::render_plain(chat::plain(n))
}

// --- Server construction. ---

pub fn sesame_server() -> SesameRocket<rocket::Build> {
    SesameRocket::build()
        .attach(Template::fairing())
        .mount("/", routes![sesame_simple, sesame_chat])
}

pub fn plain_server() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", rocket::routes![plain_simple, plain_chat])
}

// Configure Rocket before either server is built.
//
// `SesameRocket` wraps `rocket::build()` with no way to pass a custom figment,
// so the environment is the only lever that reaches both servers identically.
//
// Two settings matter. Rocket logs every request it serves, which would cost
// more than the render being measured. And `template_dir` is resolved against
// the working directory -- the workspace root under `cargo run -p` -- so it is
// pinned to an absolute path derived from the manifest location, letting the
// benchmark run correctly from anywhere.
pub fn configure() {
    let templates = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
    std::env::set_var("ROCKET_TEMPLATE_DIR", templates);
    std::env::set_var("ROCKET_LOG_LEVEL", "off");
    std::env::set_var("ROCKET_CLI_COLORS", "false");
}

// URI for the chat endpoint at a given message count. Both servers mount the
// same paths, so one builder serves both.
pub fn chat_uri(n: usize) -> String {
    format!("/bench/chat/{}", n)
}

pub const SIMPLE_URI: &str = "/bench/simple";
