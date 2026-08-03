# mini-bench

Standalone micro-benchmarks for isolating the cost of individual Sesame
mechanisms. Each subdirectory is its own crate (a workspace member) that can be
run independently:

```
cargo run -p <crate> --release                 # single-binary crates
cargo run -p <crate> --bin <bin> --release     # crates with several experiments
```

Micro-benchmarks here are meant to be small, focused, and reproducible: they
measure one hot path against a baseline, print a table, and validate that the
variants they compare produce identical results.

| crate             | measures                                                       |
|-------------------|----------------------------------------------------------------|
| `schema_policy`   | constructing and attaching schema policies to DB query results |
| `template_render` | building a template's context: `PConTemplate` vs plain Rocket  |

A crate with more than one experiment keeps its payloads, policies, data, and
templates in a shared `common` module, so its binaries measure the same work at
different scopes rather than approximations of each other. `template_render`
does this: `standalone` times the render call alone, and `endpoint` times a full
request round-trip over the same payloads.
