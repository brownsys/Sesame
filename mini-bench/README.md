# mini-bench

Standalone micro-benchmarks for isolating the cost of individual Sesame
mechanisms. Each subdirectory is its own binary crate (a workspace member) that
can be run independently:

```
cargo run -p <crate> --release
```

Micro-benchmarks here are meant to be small, focused, and reproducible: they
measure one hot path against a baseline, print a table, and validate that the
variants they compare produce identical results.

| crate           | measures                                                        |
|-----------------|-----------------------------------------------------------------|
| `schema_policy` | constructing and attaching schema policies to DB query results  |
