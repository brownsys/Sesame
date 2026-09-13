# Running Scrutinizer

Scrutinizer checks that the closures passed to `VerifiedRegion::new` are pure.
Both apps in this workspace pass: **voltron has 4 regions, youchat has 3**.

## Install

```bash
git clone https://github.com/brownsys/scrutinizer <scrutinizer_dir>
cd <scrutinizer_dir>
./scripts/scrutinizer-install
```

## Run

Each app has its own (identical) `scrutinizer-config.toml`; run from its
directory.

```bash
cd applications/voltron/server     # or: cd applications/youchat
cargo +nightly-2023-08-25 scrutinizer --config-path scrutinizer-config.toml
```

Keep the `+nightly-2023-08-25`: `rust-toolchain.toml` pins a newer toolchain and
you get a driver mismatch without it. No `cargo clean` needed -- scrutinizer
builds into its own `target/plugin-nightly-2023-08-25/`.

**After editing any source, clear the incremental cache first.** This is the most
important step. rustc reuses cached MIR for unchanged items, so `mir_borrowck`
never re-runs for them and scrutinizer panics with `mir_borrowck override should
have stored body for item: ...`. A `touch` is not enough:

```bash
T=target/plugin-nightly-2023-08-25/x86_64-unknown-linux-gnu/debug
CRATE=voltron    # or: youchat
rm -rf $T/incremental $T/deps/*$CRATE* $T/.fingerprint/$CRATE-*
```

## Output

Results go to the **workspace root** (this directory), one timestamped
`*.scrutinizer.result.json` per crate per run; they accumulate, so take the
newest with a non-empty `results` array and the `"crate_name"` you ran.

Exit 0 = all pure. Exit 101 = either a rejected region or a build failure, so
parse the JSON rather than trusting the code. A rejected region has
`"status": false`, and its `failing` array is the call chain that made it impure.

```bash
python3 -c '
import json,sys
d=json.load(open(sys.argv[1]))
for r in d["results"]:
    if not r["status"]:
        print(r["def_id"])
        for f in r["failing"]:
            fn=dict(f)["function"]
            print("  ", fn["def_id"], "| unhandled:", fn.get("unhandled"),
                  "| has_body:", fn.get("has_body"))
' "$(ls -t *.scrutinizer.result.json | head -1)"
```

## Verify it still works

A broken setup reports a clean run, so confirm it can still reject something.
Add `println!("{}", email);` inside the `email_from_token` region in
`applications/voltron/server/src/routes/common.rs`, clear the incremental cache
as above, and re-run.

Expected: exit **101**, and `email_from_token::{closure#0}` with
`"status": false`, `passing: 148`, `failing: 20` -- the chain
`_print -> print_to -> write_fmt -> fmt::write -> Write::write_str`, impure
because of the global `Stdout` handle and two unresolvable dynamic dispatches.

Remove the `println!`, clear the cache again, re-run: 4 regions `true`,
`email_from_token` back to `passing: 3`, exit 0.

## Gotchas

**Silent success.** Several failure modes exit 0 while analyzing nothing. Always
check the region count matches the number of `VerifiedRegion::new` sites in the
source (voltron 4, youchat 3).

- *Warm cache*: if cargo compiles nothing, no result file is written at all.
- *`results: []`*: means no regions were found, not that they passed. Detection
  matches a hardcoded path in `scrutils/src/selector/ppr.rs`; upstream still has
  the pre-rename `alohomora::pure::PrivacyPureRegion::<F>::new`, current is
  `sesame::verified::VerifiedRegion::<F>::new`. Any future rename breaks this
  silently.
- *Stale dumps*: an incremental run can analyze a body from a previous edit. The
  span in `failing` and the `passing` count are the tells -- check they match the
  source you actually have.

**Whole workspace builds.** Scrutinizer uses `CrateFilter::AllCrates`, so a bare
`cargo check` builds every member; breakage in a crate the app under analysis
does not even use still stops the run.

**Dependency pins.** `Cargo.lock` must stay pinned for the 1.74 nightly;
`cargo update` undoes these and the failures return.

| Crate | Pin | Why |
|---|---|---|
| `deadpool-runtime` | 0.1.3 | 0.1.4 needs rustc >= 1.75 |
| `serde`, `serde_derive` | 1.0.195 | later versions add `Saturating` impls |
| `serde_json` | 1.0.140 | 1.0.145 needs serde >= 1.0.220 |
| `slog` | 2.7.0 | 2.8.2 pulls `serde_core` |
| `pem` (3.x) | 3.0.5 | 3.0.6 pulls `serde_core` |
| `sqlx` | 0.7.3 | 0.7.4 uses `async fn` return-type projections |

serde is the subtle one: its build script sets `no_core_num_saturating` only when
rustc minor < 74, but this nightly reports 74 while predating the actual
stabilization. `RUSTFLAGS` does not help -- cargo does not apply it to these
dependency units. Pin below the impls.
