// Experiment 2: the cost of a Sesame render over a full request round trip.
//
// Times an HTTP request against a Sesame endpoint versus the identical endpoint
// on a plain Rocket server. Both handlers do exactly one thing -- look up a
// prebuilt payload and render it -- so the difference between the two is the
// render path plus whatever Sesame adds to request handling, and nothing else.
//
// This is the same work experiment 1 measures, wrapped in everything a real
// request pays for: routing, guards, the template engine, and response
// construction. Rocket's fixed per-request cost dilutes the ratio, so read this
// table alongside experiment 1's rather than on its own -- the standalone
// numbers say what the render costs, these say what fraction of a served page
// that amounts to.
//
// Payloads, policies, data, and templates are shared with experiment 1 via
// `common`, and the servers live in `server` so the data stays independent of
// how it is served.

use std::time::Duration;

use rocket::http::Status;
use rocket::local::blocking::Client;

use sesame_rocket::testing::SesameClient;

use template_render::common::{chat, simple, timing};
use template_render::server;

// Round trips are slower and jitterier than a bare render, so spend longer per
// measurement than the standalone experiment does.
const BUDGET: Duration = Duration::from_secs(5);

fn main() {
    // Must happen before either server is built.
    server::configure();

    simple::setup();
    chat::setup();

    let sesame = SesameClient::untracked(server::sesame_server()).expect("sesame client");
    let plain = Client::untracked(server::plain_server()).expect("plain client");

    println!("Experiment 2: end-to-end request round trip");
    println!("===========================================\n");
    println!("A GET against an endpoint whose handler does nothing but render a");
    println!("prebuilt payload. Includes routing, guards, the template engine,");
    println!("and response construction.\n");

    validate(&sesame, &plain);
    report_simple(&sesame, &plain);
    report_chat(&sesame, &plain);
}

// Fetch a URI, asserting the request succeeded, and return the response body.
fn fetch(client: &Client, uri: &str) -> String {
    let response = client.get(uri).dispatch();
    assert_eq!(response.status(), Status::Ok, "{} did not return 200", uri);
    response.into_string().expect("response had no body")
}

// Confirm the two servers agree before timing anything. The Sesame path reaches
// the template engine as a figment Value built from a Renderable tree, the
// plain path as a serialized struct; if those diverge, the two servers are not
// doing comparable work and the timings below would be meaningless.
fn validate(sesame: &Client, plain: &Client) {
    let uris: Vec<String> = std::iter::once(String::from(server::SIMPLE_URI))
        .chain(chat::NS.iter().map(|&n| server::chat_uri(n)))
        .collect();

    for uri in &uris {
        let s = fetch(sesame, uri);
        let p = fetch(plain, uri);
        assert_eq!(
            s, p,
            "sesame and plain rendered different HTML for {}\n--- sesame ---\n{}\n--- plain ---\n{}",
            uri, s, p
        );
    }
    println!(
        "Validated: both servers return byte-identical HTML for all {} endpoints.\n",
        uris.len()
    );
}

fn report_simple(sesame: &Client, plain: &Client) {
    let uri = server::SIMPLE_URI;
    let plain_t = timing::time_with(BUDGET, || fetch(plain, uri));
    let sesame_t = timing::time_with(BUDGET, || fetch(sesame, uri));

    println!("simple payload: 1 plain field + 2 PCon<_, NoPolicy> fields\n");
    println!(
        "  {:>11} {:>11} {:>9} {:>11}",
        "plain (us)", "sesame (us)", "ratio", "delta (us)"
    );
    println!("  {:->11} {:->11} {:->9} {:->11}", "", "", "", "");
    println!(
        "  {:>11.3} {:>11.3} {:>8.2}x {:>11.3}",
        plain_t.avg_us,
        sesame_t.avg_us,
        sesame_t.avg_us / plain_t.avg_us,
        sesame_t.avg_us - plain_t.avg_us,
    );
    println!(
        "\n  ({} reps plain, {} reps sesame)\n",
        plain_t.reps, sesame_t.reps
    );
}

fn report_chat(sesame: &Client, plain: &Client) {
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
        let uri = server::chat_uri(n);
        let plain_t = timing::time_with(BUDGET, || fetch(plain, &uri));
        let sesame_t = timing::time_with(BUDGET, || fetch(sesame, &uri));

        let leaves = chat::leaves(n);
        let delta_us = sesame_t.avg_us - plain_t.avg_us;
        println!(
            "  {:>5} {:>8} {:>12.3} {:>12.3} {:>8.2}x {:>12.3} {:>10.1}",
            n,
            leaves,
            plain_t.avg_us,
            sesame_t.avg_us,
            sesame_t.avg_us / plain_t.avg_us,
            delta_us,
            delta_us * 1000.0 / leaves as f64,
        );
        min_reps = min_reps.min(plain_t.reps).min(sesame_t.reps);
    }

    println!("\n  leaves  = protected leaves the render must policy-check (3N + 2).");
    println!("  delta   = absolute Sesame overhead over the plain Rocket path.");
    println!("  ns/leaf = that overhead amortised per protected leaf.");
    println!(
        "  Each measurement fills a ~{}s budget; the smallest run was {} reps.",
        BUDGET.as_secs(),
        min_reps
    );
    println!("\n  Ratios here are diluted by Rocket's fixed per-request cost; compare");
    println!("  the delta column against experiment 1 to see how much of the render");
    println!("  overhead survives a full round trip.");
}
