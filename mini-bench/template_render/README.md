# template_render micro-benchmark

Measures the overhead Sesame introduces when rendering HTML templates —
`PConTemplate::render` against plain Rocket's
`rocket_dyn_templates::Template::render`.

Two experiments share one set of payloads, policies, data, and templates, so
they measure literally the same work at two different scopes:

| binary       | scope                                                        |
|--------------|--------------------------------------------------------------|
| `standalone` | the render call alone — no HTTP, no engine, no database      |
| `endpoint`   | a full request round-trip against an endpoint that renders   |

```
cargo run -p template_render --bin standalone --release
cargo run -p template_render --bin endpoint --release
```

Read the two together. The standalone table says what the render costs; the
endpoint table says what fraction of a served page that amounts to, and whether
any overhead appears outside the render call itself.

## What the Sesame path does extra

In the standalone experiment both calls stop at the same point — a built context
value, ready for the template engine. Rocket defers the engine pass to the
responder, so neither variant renders HTML there; the difference is purely the
cost of preparing a template's data.

Plain Rocket does one thing: `serde_json::to_value(&context)`, serializing the
struct straight into the `serde_json::Value` that `Template` stores
(`lib.rs:328` in the Rocket fork, where `to_value` and `Value` are
`serde_json`'s — see `lib.rs:170`).

Sesame does three:

1. **`params.render()`** — builds a `Renderable` tree. The `PConRender` derive
   emits one `BTreeMap` per struct and a freshly allocated `String` for every
   field key, on every call.
2. **`.transform(..)`** — walks the tree, performing one policy check and one
   `erased_serde` dispatch per protected leaf, producing a **figment** `Value`.
3. **`Template::render(name, value)`** — serializes that figment `Value` into a
   `serde_json::Value` (`sesame/rocket/src/rocket/template.rs:33`).

So Sesame materializes three full intermediate trees where plain Rocket
materializes one, and the figment tree exists only to be immediately converted
into a different library's tree.

Note what this does **not** mean: both paths hand the template engine a
`serde_json::Value`, since that is what `Template` stores either way. The engine
pass is identical work in both variants, and the figment `Value` never reaches
handlebars.

## The payloads

**simple** — the floor: `{ plain: String, text: PCon<String, NoPolicy>,
number: PCon<i32, NoPolicy> }`. Tiny tree, trivial policy check, so what it
costs is fixed overhead rather than anything that scales.

**chat** — modelled on youchat's `ChatContext`: two protected header fields, a
plain `bool`, and `N` messages each carrying three protected fields plus a
plain `timestamp`. The unprotected fields are deliberate — they exercise the
`Serialize` fallback arm of the render tree alongside the `PCon` arm.

Every protected leaf carries `TemplatePolicy`, which stores the name of the
template it will accept and compares it against the
`Reason::TemplateRender(name)` that `PConTemplate::render` threads down to each
leaf. So a render performs `3N + 2` real string-comparing policy checks against
live plumbing, not a stub the optimizer can fold away.

`N` sweeps `{0, 20, 40, 60, 80, 100, 500, 1000}`. `N = 0` isolates fixed cost;
20–100 is the realistic range for a chat page; 500 and 1000 extend the sweep far
enough for the per-leaf slope to stand clear of the noise floor, which is where
the effect of an optimization to this path will show up first.

Payloads are built once during `setup()` and stored in a map keyed by `N`.
Construction allocates a string and a policy per leaf; that cost belongs to
setup, so it never happens inside a timed region.

## Experiment 2: the endpoint

`server.rs` builds two servers — one `SesameRocket`, one plain `Rocket` —
exposing the same two paths, `/bench/simple` and `/bench/chat/<n>`. Each handler
does exactly one thing: look up a payload `setup()` prebuilt and hand it to the
matching render helper from `common`. No database, no computation, nothing that
lands in the measurement but the render. Requests go through
`rocket::local::blocking::Client`; `SesameClient` is a thin `Deref` wrapper over
that same type, so both variants use identical client machinery and the
difference is entirely server-side.

Two asymmetries are real and left in rather than papered over. Sesame routes
receive path parameters already wrapped, so the chat handler pays a
`discard_box` on `n` that the plain handler does not (a move, no policy check).
And the Sesame handler takes a `Context<()>` request guard that the plain one
has no analogue for. Both are fixed per request, so they show up in the `N = 0`
row and not in the slope.

Rocket is configured through the environment (`server::configure()`), because
`SesameRocket::build()` wraps `rocket::build()` with no figment injection point
and the environment is the only lever reaching both servers identically. It
turns request logging off — Rocket logs every request, which would cost more
than the render being measured — and pins `template_dir` to an absolute path, as
the default resolves against the working directory, which is the workspace root
under `cargo run -p`.

## Methodology

Nothing here touches I/O — every measured operation is in-process, so the
optimizer is free to delete a call whose result goes unused. Every operation
therefore passes through `black_box`.

Cost spans three orders of magnitude across the sweep (3 leaves vs. 3002), so a
fixed repetition count would be noise at one end and take minutes at the other.
A short warmup doubles as a cost probe, and the repetition count is chosen from
it to fill a fixed time budget per measurement: 2 s standalone, 5 s end-to-end,
since round trips carry more jitter.

Both experiments check they are on the success path before timing. Standalone
asserts each render returns `Ok` — a failed policy check returns early, and
timing that would measure the error path. The endpoint experiment asserts a 200
and **byte-compares the two servers' HTML** at every endpoint before timing
anything. Both paths reach the engine as a `serde_json::Value`, but they arrive
by different routes — via a `Renderable` tree and a figment `Value` on one side,
straight from the struct on the other — so if the results diverged the two
servers would not be doing comparable work. They do not diverge, despite
`Renderable::Dict` being a `BTreeMap` (alphabetical) against serde's declaration
order, since handlebars resolves by name.

Two things sit inside the standalone Sesame timed region worth naming. The
`Context` is constructed per call (`Context::empty()`), since
`PConTemplate::render` consumes one — but it allocates nothing, and
`ExtensionContext::new` boxes only a zero-sized `Option<()>`. And the Sesame
path returns a `Result` the plain path doesn't, so it pays a discriminant check.
Both are noise-level next to a ~200 ns/leaf effect.

## Baseline results

Measured before any optimization of the render path, for later comparison.
Absolute numbers are machine-specific; the ratios and ns/leaf are the durable
part.

### Experiment 1 — standalone

```
simple payload: 1 plain field + 2 PCon<_, NoPolicy> fields

   plain (us) sesame (us)     ratio  delta (us)
        0.104       0.502     4.83x       0.398

chat payload:

      N   leaves   plain (us)  sesame (us)     ratio   delta (us)    ns/leaf
      0        2        0.138        0.634     4.60x        0.496      248.1
     20       62        2.946       15.215     5.16x       12.269      197.9
     40      122        8.684       35.185     4.05x       26.501      217.2
     60      182       12.955       51.469     3.97x       38.513      211.6
     80      242       16.908       68.620     4.06x       51.712      213.7
    100      302       21.457       85.279     3.97x       63.822      211.3
    500     1502      101.983      424.476     4.16x      322.493      214.7
   1000     3002      205.569      870.688     4.24x      665.119      221.6
```

Sesame costs about **5x** plain Rocket on tiny payloads, settling to a steady
**~4.1x** from `N=40` up, with a marginal cost of roughly **~215 ns per
protected leaf**. (Per-message cost dips at `N=20` in *both* variants, so that
is a small-payload allocator/cache effect, not a Sesame artifact.)

The important part is that the ratio does not decay with size: Sesame's overhead
is *proportional* to the work plain Rocket does, not a fixed startup cost that
amortises away, so a bigger page pays proportionally more.

### Experiment 2 — end-to-end

```
simple payload: 1 plain field + 2 PCon<_, NoPolicy> fields

   plain (us) sesame (us)     ratio  delta (us)
        2.594       3.095     1.19x       0.501

chat payload:

      N   leaves   plain (us)  sesame (us)     ratio   delta (us)    ns/leaf
      0        2        2.888        3.510     1.22x        0.622      311.2
     20       62       27.754       43.343     1.56x       15.589      251.4
     40      122       51.715       81.793     1.58x       30.078      246.5
     60      182       76.982      124.490     1.62x       47.508      261.0
     80      242      101.428      162.986     1.61x       61.559      254.4
    100      302      126.316      203.147     1.61x       76.831      254.4
    500     1502      617.155      984.451     1.60x      367.296      244.5
   1000     3002     1236.591     1986.919     1.61x      750.328      249.9
```

Over a real request, a Sesame page costs **~1.6x** a plain Rocket one, steady
from `N=20` up. The ratio is far below experiment 1's 4.1x only because the
engine pass and HTTP handling — identical work in both variants — dilute it.

### Reading the two together

The `delta` column is the one to compare, and it does **not** carry over
unchanged:

| N    | render delta | round-trip delta | ns/leaf (render → round-trip) |
|------|--------------|------------------|-------------------------------|
| 100  | 63.8 µs      | 76.8 µs          | 211 → 254                     |
| 1000 | 665.1 µs     | 750.3 µs         | 222 → 250                     |

About **85–88% of the end-to-end overhead is the render call itself**, which is
what experiment 1 measures and what optimizing this path will move.

The remaining ~12–15% appears *outside* the render call and scales with leaf
count, so it is not the fixed per-request cost of the guard and `discard_box`
(those are flat, and visible in the `N = 0` row: 0.50 µs → 0.62 µs). At `N=1000`
the non-render portion of a request is ~1025 µs plain against ~1124 µs Sesame.

**This gap is not explained.** It is specifically *not* the template engine:
`Template` stores a `serde_json::Value` in both variants, so handlebars receives
the same type and does the same work either way.

The leading hypothesis is measurement context rather than different work. A
Sesame render allocates far more transient memory — N `BTreeMap`s with 4N owned
`String` keys, a figment `Value` tree, and a `serde_json::Value` tree, against
plain's single tree. In experiment 1's tight loop the allocator reuses hot
blocks and everything stays cached; inside a request those allocations interleave
with Rocket's async machinery and response buffers, so identical logical work
costs more wall-clock. The `N=20` dip visible in *both* variants of experiment 1
shows this benchmark is sensitive to such effects at the 20–30% level, which is
the size of this gap.

Two ways to settle it, neither done yet: time the engine pass in isolation
(build both `Template`s, then call `Template::show` on each and subtract), which
would confirm directly that the engine is symmetric; or compare per-request
allocation counts between the two servers.

Either way, the optimization target is unchanged — the ~86% measured by
experiment 1, in the three steps above.

## Optimization plan

The numbers above are the pre-optimization baseline. The plan attacks the two
intermediate representations Sesame builds that plain Rocket does not, in
increasing order of blast radius, re-benchmarking after each step.

### Step 1 — remove the redundant serialization

Have `transform` produce a `serde_json::Value` directly instead of a figment
`Value`, and add a constructor to the Rocket fork that takes that value as-is
(`Template::from_json(name, value)` or similar), so `PConTemplate::render` stops
round-tripping a fully built tree through a second serialization.

The alternative considered and rejected was implementing `Serialize` on a
private wrapper around `Renderable` and letting Rocket serialize it. Two reasons
it loses:

- `Template::render` discards serialization errors (`value: to_value(context).ok()`,
  `lib.rs:328`), so a policy denial would surface as a 500 "failed to serialize"
  rather than the current 491, indistinguishable from a genuine bug — and
  `PConTemplate::render` could no longer return `SesameRenderResult` at all,
  since the check would happen after it returns.
- It would put the timing of a security-critical check under Rocket's control.
  It is eager today only because `Template::render` calls `to_value` eagerly; if
  that ever moved to the responder, policy checks would silently follow.

Keeping the check inside `transform` keeps both the error type and the check
timing where Sesame controls them.

### Step 2 — flatten the render tree

Change `Renderable::Dict` from `BTreeMap<String, Renderable<'a>>` to
`Vec<(&'a str, Renderable<'a>)>`, and update the derive to emit borrowed keys
with no allocation. Nothing looks a key up — the dict is only ever iterated — so
the map is paying for a capability that is never used.

Details that matter:

- **`&'a str`, not `&'static str`.** The derive's keys are literals, but the
  `HashMap` impls' keys are not; the shared lifetime lets those borrow
  (`key.as_str()`, `*key`) instead of cloning, and avoids needing two dict
  representations. There is no runtime difference between the two lifetimes.
- **Delete the redundant `k.clone()`** in `transform` (`render.rs:64`).
  `transform` consumes `self`, so iterating the map already yields owned
  `String`s; the clone allocates a second copy of every key and drops the
  original. Independent of everything else here.
- **Key allocations per struct of `k` fields:** 3k today → 2k after step 1 → k
  after dropping the clone, which is parity with plain Rocket (serde emits one
  `String` per field into `serde_json::Map`). Step 2 does not reduce this
  further, because the output map is keyed by `String` regardless. What it buys
  is deleting a whole `BTreeMap` per struct in favour of one
  `Vec::with_capacity(k)` — with `k` a compile-time constant in the derive — and
  removing the O(log k) *string comparisons* each `BTreeMap::insert` performs.
- **Ordering.** The `BTreeMap` currently normalizes `HashMap`'s nondeterministic
  iteration order. Going all-Vec moves that normalization to the output
  container, which is fine for default `serde_json` (a `BTreeMap`) but would
  make `HashMap`-sourced output non-reproducible if anything ever enabled
  `preserve_order`. Sorting in the two `HashMap` impls is cheap insurance.
- Breaking change to a public enum. Touch points: `Renderable::Dict`, the three
  derive sites in `derive/src/render.rs`, the two `HashMap` impls, and the
  `transform` arm.

### Step 3 — re-measure and reiterate

Re-run both experiments and compare against the baseline tables above. `ns/leaf`
in experiment 1 is the number to watch; experiment 2 says how much of any win
reaches a served page.

The target is to get experiment 1's overhead down to roughly the cost of the
policy checks themselves. If steps 1 and 2 do not get close, the next move is to
attribute what remains rather than guess: widening `Renderable::transform` from
`pub(crate)` to `pub` lets experiment 1 report `render` / `transform` /
`to_value` as separate columns, turning the single ~215 ns/leaf figure into a
per-step breakdown.

Known to survive all three steps: the `erased_serde` dynamic dispatch per leaf,
and the `Renderable` tree itself. Eliminating the latter would mean a
serde-driven `PConRender` whose derive writes fields straight into a serializer
and never materializes an intermediate tree — a larger redesign, worth deciding
on only once the cheaper steps have been measured.

## Knobs that would bias the result

Fixed here at realistic values, and worth remembering before reading too much
into the headline ratio:

- **String length.** Longer `content` fields would let serialization dominate
  and make Sesame's overhead look proportionally smaller. Messages are ~48
  characters.
- **Leaf density.** The fraction of fields that are `PCon`-wrapped. The chat
  payload is 3-of-4 per message.
- **Nesting depth.** Both payloads are shallow, matching youchat.
