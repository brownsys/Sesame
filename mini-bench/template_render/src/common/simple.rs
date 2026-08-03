// The minimal payload: three fields, one plain and two protected with
// `NoPolicy`. This is the floor -- what a render costs when the tree is tiny
// and the policy check is trivial, so whatever the Sesame path costs here is
// fixed overhead rather than anything that scales with the data.

use std::sync::OnceLock;

use rocket_dyn_templates::Template;
use serde::Serialize;

use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_rocket::error::SesameRenderResult;
use sesame_rocket::render::PConRender;
use sesame_rocket::rocket::PConTemplate;

// Name of the template this payload renders into. Both paths use it, so both
// engines do identical work downstream.
pub const TEMPLATE: &str = "simple";

#[derive(PConRender)]
pub struct SimpleContext {
    pub plain: String,
    pub text: PCon<String, NoPolicy>,
    pub number: PCon<i32, NoPolicy>,
}

// The same payload with the PCons stripped -- what plain Rocket renders.
#[derive(Serialize)]
pub struct PlainSimpleContext {
    pub plain: String,
    pub text: String,
    pub number: i32,
}

// The data. Identical in both variants, so the two paths serialize the same
// bytes into the same shape.
const PLAIN_FIELD: &str = "Sesame template render benchmark";
const TEXT_FIELD: &str = "a short protected string";
const NUMBER_FIELD: i32 = 42;

static SESAME: OnceLock<SimpleContext> = OnceLock::new();
static PLAIN: OnceLock<PlainSimpleContext> = OnceLock::new();

// Build both payloads once, up front. Construction allocates (a `String` per
// field, plus a policy per protected leaf) and that is not what we are
// measuring, so it must happen outside any timed region.
pub fn setup() {
    SESAME.get_or_init(|| SimpleContext {
        plain: String::from(PLAIN_FIELD),
        text: PCon::new(String::from(TEXT_FIELD), NoPolicy {}),
        number: PCon::new(NUMBER_FIELD, NoPolicy {}),
    });
    PLAIN.get_or_init(|| PlainSimpleContext {
        plain: String::from(PLAIN_FIELD),
        text: String::from(TEXT_FIELD),
        number: NUMBER_FIELD,
    });
}

pub fn sesame() -> &'static SimpleContext {
    SESAME.get().expect("common::simple::setup() not called")
}

pub fn plain() -> &'static PlainSimpleContext {
    PLAIN.get().expect("common::simple::setup() not called")
}

// The two render paths under test. The endpoint experiment calls these too, so
// both experiments measure the same call.

pub fn render_sesame(
    payload: &SimpleContext,
    context: Context<()>,
) -> SesameRenderResult<PConTemplate> {
    PConTemplate::render(TEMPLATE, payload, context)
}

pub fn render_plain(payload: &PlainSimpleContext) -> Template {
    Template::render(TEMPLATE, payload)
}
