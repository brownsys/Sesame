# schema_policy micro-benchmark

Measures the cost of constructing schema policies and attaching them to the
cells of a DB query result — the bottleneck addressed by commit `affd3ec`
("perf(mysql): resolve schema policies once per query instead of per cell").

## What it compares

The same flow — `SELECT k` rows, iterate the result set materialising every
cell, then in Rust count the rows and sum one column and reveal that sum — is
run against three strategies:

- **plain** — raw `mysql` connector, no Sesame policies. The floor.
- **old** — the pre-commit behaviour, reconstructed by calling the still-public
  `sesame_mysql::get_schema_policies` once per cell (one `RwLock` read + map
  lookup + owned-`String` allocation per cell of every row).
- **new** — the current `sesame_mysql` path: `SesameConn::query_iter` snapshots
  the policy factories for the column set once per query and shares that
  snapshot across all rows.

`old` and `new` both register the same `ScorePolicy` (via `#[schema_policy]`)
on the `score` column; its constructor pulls the row's `owner` value into the
policy struct to simulate a policy whose decision depends on row data. Because
registration is process-global, one registration serves both Sesame variants.

The benchmark asserts all three variants agree on `(row count, summed column)`
at every `k`, so any per-cell / per-query divergence would fail the run.

## Running

Requires a MySQL/MariaDB server. Defaults to
`mysql://root:password@127.0.0.1`; override with `DATABASE_URL`. It creates and
populates a throwaway `sesame_bench` database.

```
cargo run -p schema_policy --release
```

Output is a table of average latency (microseconds) per query+iterate+consume,
for `k in {1, 10, 100, 1000}`, plus:

- `old/new` — speedup delivered by the fix.
- `new/plain` — residual Sesame overhead over the no-policy floor (PCon boxing,
  policy construction, reveal) that the fix does not target.

At small `k` the fixed query round-trip dominates and the per-cell cost is
invisible; the fix's effect grows with result-set size, which is where the
per-cell registry lookups accumulated.
