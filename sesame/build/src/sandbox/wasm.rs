use crate::SesameBuilder;

// Generates wrappers.
pub fn build_library_wasm(builder: &SesameBuilder) -> String {
    builder.logger.warn("WASM", "Building WASM package");

    // Find all the features we are compiled with.
    let features: String = std::env::vars()
        .filter_map(|(var, _)| {
            if var.starts_with("CARGO_FEATURE_") {
                let feature = &var["CARGO_FEATURE_".len()..];
                Some(feature.to_lowercase())
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join(",");
    builder
        .logger
        .info("WASM", &format!("Building using features '{}'", features));

    let status = builder
        .command("Build WASM", &builder.env.cargo)
        .arg("+nightly-2023-10-06")
        .arg("build")
        .arg("--release")
        .arg("--lib")
        .args(["--features", &features])
        .args(["-Z", "build-std=std,panic_abort"])
        .args(["--target", "wasm32-rlbox.json"])
        .args([
            "--target-dir",
            &format!("{}/{}", builder.env.package_directory, "wasm_target"),
        ])
        .arg("--verbose")
        .env_remove("CARGO")
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("RUSTC")
        .env_remove("RUSTDOC")
        .env_remove("RUSTFLAGS")
        .env_remove("RUSTFLAGS")
        .env_remove("RUSTCFLAGS")
        .env_remove("RUST_LOG")
        .execute()
        .expect("Failed to build sandboxes library with wasm");
    if !status.success() {
        builder
            .logger
            .error("WASM", "Failed to build sandboxes library with wasm");
        panic!("");
    }

    let path = format!(
        "{}/{}/{}",
        builder.env.working_directory,
        "wasm_target/wasm32-rlbox/release",
        builder.env.lib_name()
    );

    builder
        .logger
        .info("WASM", &format!("WASM module built at '{}'", path));
    path
}
