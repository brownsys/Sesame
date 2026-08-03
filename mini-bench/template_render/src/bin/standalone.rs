// Experiment 1: the cost of building a template context, standalone.
//
// Times `PConTemplate::render` against `rocket_dyn_templates::Template::render`
// with nothing else in the timed region -- no database, no HTTP, no template
// engine. Both calls stop at the same point: a built context value, ready for
// the engine. Neither actually renders HTML (Rocket defers that to the
// responder), so the two are directly comparable and the difference is exactly
// what Sesame adds to preparing a template's data.
//
// What the Sesame path does that the plain path does not:
//   1. `params.render()`   -- builds a `Renderable` tree; one BTreeMap per
//                             struct, with a freshly allocated String per key.
//   2. `.transform(..)`    -- one policy check and one erased_serde dispatch
//                             per protected leaf, producing a figment Value.
//   3. `Template::render`  -- serializes that figment Value into a figment
//                             Value again, a redundant full-tree round trip
//                             the plain path never pays.
//
// The sweep is designed to separate fixed cost from per-leaf cost: the simple
// payload and chat at N=0 show the former, the chat sweep's slope the latter.
//
// Payloads, policy, and data come from `common`, and are built once during
// setup so that no allocation of benchmark data happens inside a timed region.

use std::hint::black_box;

use sesame::context::Context;

use template_render::common::{chat, simple, timing};

fn main() {
    simple::setup();
    chat::setup();
    sanity_check();

    println!("Experiment 1: standalone template-context construction");
    println!("======================================================\n");
    println!("PConTemplate::render vs Template::render. Neither renders the");
    println!("template; both stop at the built context value.\n");

    report_simple();
    report_chat();
}

// Confirm both paths are on their success path before timing anything. A failed
// policy check returns early, so timing it would measure the error path rather
// than a full render.
fn sanity_check() {
    simple::render_sesame(simple::sesame(), Context::empty())
        .expect("simple payload failed to render");
    for &n in chat::NS {
        chat::render_sesame(chat::sesame(n), Context::empty())
            .unwrap_or_else(|e| panic!("chat payload n={} failed to render: {:?}", n, e));
    }
}

fn report_simple() {
    let payload = simple::sesame();
    let plain_payload = simple::plain();

    let plain = timing::time(|| simple::render_plain(black_box(plain_payload)));
    let sesame =
        timing::time(|| simple::render_sesame(black_box(payload), Context::empty()).unwrap());

    println!("simple payload: 1 plain field + 2 PCon<_, NoPolicy> fields\n");
    println!(
        "  {:>11} {:>11} {:>9} {:>11}",
        "plain (us)", "sesame (us)", "ratio", "delta (us)"
    );
    println!("  {:->11} {:->11} {:->9} {:->11}", "", "", "", "");
    println!(
        "  {:>11.3} {:>11.3} {:>8.2}x {:>11.3}",
        plain.avg_us,
        sesame.avg_us,
        sesame.avg_us / plain.avg_us,
        sesame.avg_us - plain.avg_us,
    );
    println!(
        "\n  ({} reps plain, {} reps sesame)\n",
        plain.reps, sesame.reps
    );
}

fn report_chat() {
    println!("chat payload: 2 PCon header fields + 1 plain field");
    println!("              + N messages x (3 PCon fields + 1 plain field)\n");
    println!(
        "  {:>5} {:>8} {:>12} {:>12} {:>9} {:>12} {:>10}",
        "N", "leaves", "plain (us)", "sesame (us)", "ratio", "delta (us)", "ns/leaf"
    );
    println!(
        "  {:->5} {:->8} {:->12} {:->12} {:->9} {:->12} {:->10}",
        "", "", "", "", "", "", ""
    );

    let mut min_reps = usize::MAX;
    for &n in chat::NS {
        let payload = chat::sesame(n);
        let plain_payload = chat::plain(n);

        let plain = timing::time(|| chat::render_plain(black_box(plain_payload)));
        let sesame =
            timing::time(|| chat::render_sesame(black_box(payload), Context::empty()).unwrap());

        let leaves = chat::leaves(n);
        let delta_us = sesame.avg_us - plain.avg_us;
        println!(
            "  {:>5} {:>8} {:>12.3} {:>12.3} {:>8.2}x {:>12.3} {:>10.1}",
            n,
            leaves,
            plain.avg_us,
            sesame.avg_us,
            sesame.avg_us / plain.avg_us,
            delta_us,
            delta_us * 1000.0 / leaves as f64,
        );
        min_reps = min_reps.min(plain.reps).min(sesame.reps);
    }

    println!(
        "\n  leaves  = protected leaves the render must policy-check (3N + 2)."
    );
    println!("  delta   = absolute Sesame overhead over the plain Rocket path.");
    println!("  ns/leaf = that overhead amortised per protected leaf.");
    println!(
        "  Each measurement fills a ~{}ms budget; the smallest run was {} reps.",
        timing::DEFAULT_BUDGET.as_millis(),
        min_reps
    );
}
