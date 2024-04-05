# Alohomora's linting library

This contains all the `dylints` Alohomora requires to ensure that application code is safe and respects
various assumptions Alohomora makes.

## Dependencies
Developers must manually install and `cargo-dylint` `rust-link` in order to use lints.
```bash
cargo install cargo-dylint --version 2.5.0
cargo install dylint-link --version 2.5.0
```

## Adding a new lint

### Lint declaration

Every lint (or group of logically related lints) should be in its own mod file under `src/`.
Inside that mod, each lint must be declared using:

```rust
declare_alohomora_lint!(
    /// ### What it does
    /// < SPECIFY WHAT THE LINT DOES >

    /// ### Why is this bad?
    /// < WHAT'S WRONG WITH THE PATTERNS DISALLOWED BY LINT >

    /// ### Example
    /// < EXAMPLE OF BAD CODE >
    /// Use instead:
    /// < EXAMPLE OF GOOD CODE >
    pub <LINT_NAME>,
    Deny, // does not allow ignoring lint via command line
    "BRIEF DESCRIPTION OF LINT",
    <function_name(function_args)>
);
```

For each declared lint, the mod must contain a `check_*` method from the [LateLintPass](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/trait.LateLintPass.html) trait, excluding the `*mut self` argument. 
To implement the trait, which has this signature: `check_expr(&mut self, _: &LateContext<'tcx>, _: &'tcx Expr<'tcx>)`, 
the mod would contain the following:
```rust
fn check_expr(cx: &LateContext<'_>, expr: &'_ rustc_hir::Expr<'_>) {
    // ...
}
```

### Lint registration

After declaring the lint and defining its function handler, the lint must be registered by adding the following line to
the end of `src/lib.rs#register_lints`.
```rust
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    // ...
    mod_name::LintName::register(lint_store);
}
```
**NOTE: LintName must be in camel case here.**

### Lint testing

You can add tests for your lint by creating a `lint_name_[legal|illegal]_[description?].rs` under examples/.

Your test file must include a `main` (but it can be empty). You must update `Cargo.toml` to declare your example there.

If you expect the test to fail due to some linting error, you must add a `.stderr` file of the same name under `examples/`.
The content of `.stderr` must match the error text exactly (which may contain special characters).

To use whatever error is actually produced when running the test (i.e. update the expected `.stderr`
to match the observed behavior), you can overwrite the `.stderr` file with the tmp file where the test runner stored
the actual output. The filename is usually of the form `/tmp/<test_name>.stage-id.stderr`, and is displayed in the
error message of the test
`
```
Actual stderr saved to /tmp/<filename>
```

# List of existing lints

## AlohomoraType

### What it does
Denies non-library implementations of AlohomoraType. 

### Why is this bad?
Developers must derive impls of AlohomoraType to ensure integrity of policy protection.

### Example
```rust
impl AlohomoraType for BadStruct {
    // ...
}
 ```
 Use instead:
```rust
#[derive(AlohomoraType)]
#[out_type(name = "GoodStructOut", to_derive = [Debug])]
pub struct GoodStruct { 
    // ...
}
```

## AlohomoraPCR
### What it does
Warns if PrivacyCriticalRegions have invalid signatures. 

### Why is this bad?
Closures in PrivacyCriticalRegions must be signed to indicate they have been 
reviewed and do not pose privacy risks. 

An invalidated signature indicates that the PCR closure or a function
the closure calls has changed since the last signature. 

### Known problems
Functions from external crates called within the PCR are not included in the hash of the closure, 
so changes in an external crate will not invalidate the signature. 
