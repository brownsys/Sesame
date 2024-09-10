# Alohomora's linting library

This contains all the `dylints` Alohomora requires to ensure that application code is safe and respects
various assumptions Alohomora makes.

## Dependencies
Developers must manually install and `cargo-dylint` `rust-link` in order to use lints.
```bash
cargo install cargo-dylint --version 2.5.0 --locked
cargo install dylint-link --version 2.5.0 --locked
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
You should choose the `check_<...>` function that meets your lint's requirements.
For example, if your trait is best suited for [`check_expr(&mut Self, &LateContext<'a>, &'a Expr<'a>`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/trait.LateLintPass.html#method.check_expr), then you must implement a `check_expr(&LateContext<'a>, &'a Expr<'a>)` in your mod, and then use `check_expr(a: &rustc_lint::LateContext<'a>, b: &'a rustc_hir::Expr<'a>)` as the last argument to `declare_alohomora_lint!`.

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
reviewed and are believed to not pose privacy risks. 

An invalidated signature indicates that the PCR closure or a function
the closure calls has changed since the last signature. 

### Known problems 
Changing the values of statics and constants that are used within a PCR will not invalidate the signature.

Functions from external crates called within the PCR are not included in the hash of the closure, 
so changes in an external crate will not invalidate the signature. The signature on the hash of the Cargo.lock indicates that the use external dependencies within the PCR are trusted. 

## Creating a PrivacyCriticalRegion signature
Each PCR signature is unique to the closure it signs. We use ssh-keygen to sign and verify the signed file. 

To create an author or reviewer Signature, run the `alohomora_pcr` lint with an empty string in the signature field.
The lint will fail and output a file containing the hash of the MIR of the closure. 

The hash of a PrivacyCriticalRegion in a function named `data_processing` will appear in a file of the form `pcr/data_processing-{closure#0}_hash.rs`

The code snippet of the closure will appear in another file for reference. 

Now, run the sign.sh script:
<!--- Make code --->
    #!/bin/bash

    ssh-keygen -Y sign -f $1 -n file $2
    base64 -i "$2.sig" -o "$2.sig.base"

The arguments are the path to a private key linked with your Github and a text file containing the hash of the closure to sign. 
<!--- Make code --->
    ./sign.sh /Users/name/.ssh/id_ed25519 src/pcr/hash_file.rs

The `alohomora::pcr::Signature` struct takes as arguments a github username, the PCR-specific signature, and a signature on the hash of the Cargo.lock. 

Copy-paste the appropriate encrypted signature from the generated file into each Signature struct. 

```rust
Signature {
    username: "gituser", 
    signature: "LS0tLS1CRUdJTiBTU0ggU0lHTkFUVVJFLS0tLS0KVTFOSVUwbEhBQUFBQVFBQUFETUFBQUFMYzNOb0xXVmtNalUxTVRrQUFBQWd6dGJjeE9zVzlOL09Fd2c3Y3BKZ3dUQnFMNgpGazI2ZVB2Rm1ZaXpRRjM1VUFBQUFFWm1sc1pRQUFBQUFBQUFBR2MyaGhOVEV5QUFBQVV3QUFBQXR6YzJndFpXUXlOVFV4Ck9RQUFBRUNqRStac3YzcUhROG8zL1ZOVmxacVB5MzV4REI3Ti9FVkljaFB4bllXZWFqQjQ4WC9Dc1VpcG1RN0N2RHNucXkKV1REandZVHlVUThxUWJMR0VCelJzRwotLS0tLUVORCBTU0ggU0lHTkFUVVJFLS0tLS0K"}
```

Changing the source code of the PCR will invalidate the previous signature. 

Both the closure and Signatures must be instantiated inline in the PrivacyCriticalRegion declaration.
