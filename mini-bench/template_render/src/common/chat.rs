// The realistic payload, modelled on youchat's `ChatContext`: a couple of
// protected header fields plus a list of N messages, each carrying three
// protected fields and one plain one.
//
// Every protected leaf carries `TemplatePolicy`, so a render performs 3N + 2
// real policy checks. The `admin` and `timestamp` fields are deliberately left
// unprotected, exercising the `Serialize` fallback arm of the render tree
// alongside the PCon arm.

use std::collections::HashMap;
use std::sync::OnceLock;

use rocket_dyn_templates::Template;
use serde::Serialize;

use sesame::context::Context;
use sesame::pcon::PCon;
use sesame_rocket::error::SesameRenderResult;
use sesame_rocket::render::PConRender;
use sesame_rocket::rocket::PConTemplate;

use crate::common::policy::TemplatePolicy;

pub const TEMPLATE: &str = "chat";

// Message counts to sweep. 0 isolates the fixed cost (header fields and tree
// construction with an empty list); 20..100 is the realistic range for a chat
// page; 500 and 1000 extend the sweep far enough for the per-leaf slope to
// stand clear of the noise floor.
pub const NS: &[usize] = &[0, 20, 40, 60, 80, 100, 500, 1000];

#[derive(PConRender)]
pub struct Message {
    pub sender: PCon<String, TemplatePolicy>,
    pub recipient: PCon<String, TemplatePolicy>,
    pub content: PCon<String, TemplatePolicy>,
    pub timestamp: String,
}

#[derive(PConRender)]
pub struct BenchContext {
    pub user: PCon<String, TemplatePolicy>,
    pub group: PCon<String, TemplatePolicy>,
    pub admin: bool,
    pub messages: Vec<Message>,
}

// The same payload with the PCons stripped -- what plain Rocket renders.
#[derive(Serialize)]
pub struct PlainMessage {
    pub sender: String,
    pub recipient: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct PlainBenchContext {
    pub user: String,
    pub group: String,
    pub admin: bool,
    pub messages: Vec<PlainMessage>,
}

// --- Data generation. Deterministic, and identical across the two variants. ---

const USER: &str = "alice";
const GROUP: &str = "sesame-benchmarks";
const ADMIN: bool = true;

const PARTICIPANTS: &[&str] = &["alice", "bob", "carol", "dave"];

fn sender(i: usize) -> String {
    String::from(PARTICIPANTS[i % PARTICIPANTS.len()])
}

fn recipient(i: usize) -> String {
    String::from(PARTICIPANTS[(i + 1) % PARTICIPANTS.len()])
}

// A realistically short chat message. Length matters: long strings would let
// serialization dominate and make the Sesame overhead look smaller than it is.
fn content(i: usize) -> String {
    format!("message {:04}: the quick brown fox jumps over it", i)
}

fn timestamp(i: usize) -> String {
    format!(
        "2026-08-03T{:02}:{:02}:{:02}Z",
        (i / 3600) % 24,
        (i / 60) % 60,
        i % 60
    )
}

fn build_sesame(n: usize) -> BenchContext {
    let policy = || TemplatePolicy::new(TEMPLATE);
    BenchContext {
        user: PCon::new(String::from(USER), policy()),
        group: PCon::new(String::from(GROUP), policy()),
        admin: ADMIN,
        messages: (0..n)
            .map(|i| Message {
                sender: PCon::new(sender(i), policy()),
                recipient: PCon::new(recipient(i), policy()),
                content: PCon::new(content(i), policy()),
                timestamp: timestamp(i),
            })
            .collect(),
    }
}

fn build_plain(n: usize) -> PlainBenchContext {
    PlainBenchContext {
        user: String::from(USER),
        group: String::from(GROUP),
        admin: ADMIN,
        messages: (0..n)
            .map(|i| PlainMessage {
                sender: sender(i),
                recipient: recipient(i),
                content: content(i),
                timestamp: timestamp(i),
            })
            .collect(),
    }
}

// --- Prebuilt payloads, keyed by message count. ---

static SESAME: OnceLock<HashMap<usize, BenchContext>> = OnceLock::new();
static PLAIN: OnceLock<HashMap<usize, PlainBenchContext>> = OnceLock::new();

// Build every payload in `NS` once, up front. Construction allocates a string
// and a policy per leaf; that cost belongs to setup, not to a render, so it
// must not happen inside a timed region or inside a request.
pub fn setup() {
    SESAME.get_or_init(|| NS.iter().map(|&n| (n, build_sesame(n))).collect());
    PLAIN.get_or_init(|| NS.iter().map(|&n| (n, build_plain(n))).collect());
}

pub fn sesame(n: usize) -> &'static BenchContext {
    SESAME
        .get()
        .expect("common::chat::setup() not called")
        .get(&n)
        .unwrap_or_else(|| panic!("no prebuilt payload for n={} (not in NS)", n))
}

pub fn plain(n: usize) -> &'static PlainBenchContext {
    PLAIN
        .get()
        .expect("common::chat::setup() not called")
        .get(&n)
        .unwrap_or_else(|| panic!("no prebuilt payload for n={} (not in NS)", n))
}

// Number of protected leaves a render of this payload must check.
pub fn leaves(n: usize) -> usize {
    3 * n + 2
}

// --- The two render paths under test. ---
// The endpoint experiment calls these too, so both experiments measure the
// same call.

pub fn render_sesame(
    payload: &BenchContext,
    context: Context<()>,
) -> SesameRenderResult<PConTemplate> {
    PConTemplate::render(TEMPLATE, payload, context)
}

pub fn render_plain(payload: &PlainBenchContext) -> Template {
    Template::render(TEMPLATE, payload)
}
